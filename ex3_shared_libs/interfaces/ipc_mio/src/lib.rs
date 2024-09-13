/**
 * TODO: Check if IpcClient `poll_for_conn` should timeout
 * or block indefinitely, considering that servers currently
 * do not accept until they are polled
 *
 * TODO: the efficiency of poll_ipc_* is not great, what I'll need
 * to do is probably add a parameter for how long to wait since having
 * the user constantly call this function is not ideal. Having to create
 * a new poll struct just to check for a change in `POLL_TIMEOUT_MS`
 * is not as good way to do this
 *
 * An idea to improve this efficiency is to possibly make a handler
 * struct in a way similar to C. The handler struct can store the
 * poll struct so we don't have to recreate and initialize it every
 * time. We can initialize the handler once and then that will improve
 * efficiency
 */
use std::fs;
use std::io::{Error, ErrorKind, Read};
use std::os::unix::net::SocketAddr;

use mio::net::{UnixListener, UnixStream};
use mio::{Events, Interest, Poll, Registry, Token};
use std::io::Error as IoError;
use std::path::{self, Path};
use std::time::Duration;

const SOCKET_PATH_PREPEND: &str = "/tmp/fifo_socket_";
const NUM_EVENTS: usize = 1024;
const POLL_TIMEOUT_MS: u64 = 100;

pub struct IpcClientPollHandler {
    pub poll: Poll,
    pub clients: Vec<IpcClient>,
}
impl IpcClientPollHandler {
    /// Create a new handler for the clients.
    ///
    /// Note that the vector is moved out into the struct,
    /// ideally there should not be a reason to access the
    /// vector manually.
    pub fn new(clients: Vec<IpcClient>) -> Result<IpcClientPollHandler, IoError> {
        let mut poll = Poll::new()?;
        let mut token_index = 0; // token index will match the order the array came in

        let mut client_handler = IpcClientPollHandler {
            poll: poll,
            clients: clients,
        };

        for client in &mut client_handler.clients {
            client_handler.poll.registry().register(
                &mut client.stream,
                Token(token_index),
                Interest::READABLE | Interest::WRITABLE,
            )?;
            token_index += 1;
        }

        Ok(client_handler)
    }

    /// The clients will hang for a connection for a duration of `POLL_TIMEOUT_MS`
    ///
    /// returns the number of successfully connected streams
    pub fn poll_for_conn(&mut self) -> Result<usize, IoError> {
        let mut events = Events::with_capacity(NUM_EVENTS);
        let mut num_conns = 0;

        // Consider changing the timeout possibly to `None`
        self.poll
            .poll(&mut events, Some(Duration::from_millis(POLL_TIMEOUT_MS)))?;

        for event in &events {
            if event.is_writable() {
                // we can probably read/write from the stream now if it wasn't a spurious event
                let Token(index) = event.token();
                self.clients[index].connected = true;
                num_conns += 1;
            }
        }

        Ok(num_conns)
    }

    /// polls clients for data writes to a buffer and returns the number of bytes read
    /// as well as the socket path
    ///
    /// TODO: Consider calling poll_for_conn prior to poll_for_data since poll_for_data
    /// could result in disconnects
    pub fn poll_for_data(&mut self, buf: &mut [u8]) -> Result<(usize, String), IoError> {
        let mut events = Events::with_capacity(NUM_EVENTS);

        // Consider changing the timeout possibly to `None`
        self.poll
            .poll(&mut events, Some(Duration::from_millis(POLL_TIMEOUT_MS)))?;

        for event in &events {
            if event.is_readable() {
                let Token(index) = event.token();
                let client = &mut self.clients[index];

                match client.stream.read(buf) {
                    Ok(0) => {
                        println!("Client disconnected from {}", client.socket_path);
                        client.connected = false;
                    }
                    Ok(bytes_read) => {
                        return Ok((bytes_read, client.socket_path.clone()));
                    }
                    Err(e) => {
                        eprintln!("Could not read from {}\n{}", client.socket_path, e);
                    }
                };
            }
        }

        // No data was read
        Ok((0, "".to_string()))
    }
}

pub struct IpcServerPollHandler {
    pub poll: Poll,
    pub servers: Vec<IpcServer>,
}
impl IpcServerPollHandler {
    /// Create a new handler for the servers.
    ///
    /// Note that the vector is moved out into the struct,
    /// ideally there should not be a reason to access the
    /// vector manually.
    pub fn new(servers: Vec<IpcServer>) -> Result<IpcServerPollHandler, IoError> {
        let mut poll = Poll::new()?;
        let mut token_index = 0; // token index will be 2x the index of the array it came in to keep storage for streams

        let mut server_handler = IpcServerPollHandler {
            poll: poll,
            servers: servers,
        };

        for server in &mut server_handler.servers {
            server_handler.poll.registry().register(
                &mut server.listener,
                Token(token_index),
                Interest::READABLE | Interest::WRITABLE,
            )?;
            token_index += 2;
        }

        Ok(server_handler)
    }
}
pub struct IpcClient {
    pub socket_path: String,
    pub stream: UnixStream,
    connected: bool,
}
impl IpcClient {
    pub fn new(socket_name: String) -> Result<IpcClient, IoError> {
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, socket_name);
        let stream = UnixStream::connect(&socket_path)?;

        let mut client = IpcClient {
            socket_path: socket_path,
            stream: stream,
            connected: false,
        };

        Ok(client)
    }
}

pub struct IpcServer {
    pub socket_path: String,
    listener: UnixListener,
    pub stream: Option<UnixStream>,
    connected: bool,
}
impl IpcServer {
    pub fn new(socket_name: String) -> Result<IpcServer, IoError> {
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, socket_name);
        let path = Path::new(&socket_path);
        if path.exists() {
            fs::remove_file(path)?;
        }

        let address = SocketAddr::from_pathname(&socket_path)?;

        let listener = match UnixListener::bind_addr(&address) {
            Ok(sock) => sock,
            Err(e) => {
                println!("Couldn't bind: {:?}", e);
                return Err(e);
            }
        };

        let server = IpcServer {
            socket_path: socket_path,
            listener: listener,
            stream: None,
            connected: false,
        };

        // Normally a server would accept conn here - but instead we do this in the polling loop
        Ok(server)
    }
}

/// Polls IpcClient's until one of them detects something was written
/// to their stream
///
/// TODO: the efficiency of this is not great, what I'll need to do
/// is probably add a parameter for how long to wait since having the
/// user constantly call this function is not ideal. Having to create
/// a new poll struct just to check for a change in `POLL_TIMEOUT_MS`
/// is not as good way to do this
pub fn poll_ipc_clients(
    clients: &mut Vec<&mut IpcClient>,
    buf: &mut [u8],
) -> Result<(usize, String), std::io::Error> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(NUM_EVENTS);
    let mut token_index = 0;

    for client in &mut *clients {
        poll.registry().register(
            &mut client.stream,
            Token(token_index),
            Interest::READABLE | Interest::WRITABLE,
        )?;

        token_index += 1;
    }

    poll.poll(&mut events, Some(Duration::from_millis(POLL_TIMEOUT_MS)))?;

    for event in &events {
        if event.is_readable() {
            let client = &mut clients[event.token().0];
            let bytes_read = client.stream.read(buf)?;

            if bytes_read > 0 {
                println!(
                    "Received {} bytes on socket {}",
                    bytes_read, client.socket_path
                );
                return Ok((bytes_read, client.socket_path.clone()));
            }
        }
    }
    Ok((0, "".to_string()))
}

pub fn poll_ipc_server_sockets(
    servers: &mut Vec<&mut IpcServer>,
    buf: &mut [u8],
) -> Result<(usize, String), std::io::Error> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(NUM_EVENTS);
    let mut listener_token_index = 0; // listeners will be even
                                      // streams will be odd

    for server in &mut *servers {
        poll.registry().register(
            &mut server.listener,
            Token(listener_token_index),
            Interest::READABLE | Interest::WRITABLE,
        )?;

        listener_token_index += 2;
    }

    poll.poll(&mut events, Some(Duration::from_millis(POLL_TIMEOUT_MS)))?;

    for event in &events {
        if event.is_read_closed() {
            // this will only run on streams
            println!("Client has closed connection");
            let Token(t) = event.token();
            let server = &mut servers[t / 2];

            server.stream = None;
            server.connected = false;
            continue;
        } else if !event.is_readable() {
            // Nothing to report, keep looping
            continue;
        };
        match event.token() {
            // accept connection
            Token(t) if t % 2 == 0 => {
                // This is a listener
                match servers[t / 2].listener.accept() {
                    Ok((connection, _addr)) => {
                        let server = &mut servers[t / 2];

                        // Adding the stream to Server struct
                        server.stream = Some(connection);
                        server.connected = true;

                        // Adding the stream to the poll struct
                        poll.registry().register(
                            match &mut server.stream {
                                Some(stream) => stream,
                                None => {
                                    panic!("how'd you get here? Stream was value None after assignment");
                                }
                            },
                            Token(t + 1),
                            Interest::READABLE | Interest::WRITABLE,
                        )?;
                    }
                    Err(e) => {
                        println!("failed :( with error, {:?}", e);
                    }
                }
            }
            // read data
            Token(t) if t % 2 == 1 => {
                // This is a stream
                let server = &mut servers[t / 2];
                let stream = match &mut server.stream {
                    Some(stream) => stream,
                    None => continue, // indicates there is no client conn
                };

                let bytes_read = stream.read(buf)?;

                if bytes_read > 0 {
                    println!(
                        "Received {} bytes on socket {}",
                        bytes_read, server.socket_path
                    );
                    return Ok((bytes_read, server.socket_path.clone()));
                }
            }
            _ => {} // sad useless code
        }
    }

    Ok((0, "".to_string()))
}
