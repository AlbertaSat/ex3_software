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

        client.poll_for_conn()?;
        Ok(client)
    }

    /// The client will hang for a connection for a duration of `POLL_TIMEOUT_MS`
    /// afterwards it will either successfully connect to the socket or will
    /// timeout which then becomes the responsibility of the user to handle.
    fn poll_for_conn(&mut self) -> Result<(), IoError> {
        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(NUM_EVENTS);

        poll.registry().register(
            &mut self.stream,
            Token(0),
            Interest::READABLE | Interest::WRITABLE,
        )?;

        // Consider changing the timeout possibly to `None`
        poll.poll(&mut events, Some(Duration::from_millis(POLL_TIMEOUT_MS)))?;

        for event in &events {
            if event.token() == Token(0) && event.is_readable() && event.is_writable() {
                // we can probably read/write from the stream now if it wasn't a spurious event
                self.connected = true;
                return Ok(());
            }
        }

        return Err(Error::new(
            ErrorKind::TimedOut,
            format!("Connection to {:?} timed out", self.socket_path),
        ));
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
