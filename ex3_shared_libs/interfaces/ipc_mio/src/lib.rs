/**
 * TODO: Check if IpcClient `poll_for_conn` should timeout
 * or block indefinitely, considering that servers currently
 * do not accept until they are polled
 *
 * TODO: in poll_for_data, go through all the data before
 * breaking since we might cause an issue if we're checking
 * the same events, also figure out how events work exactly.
 * This issue occurs since events are automatically reset
 * on each poll call, consider adding a queue data structure
 * to store the event. This way we can handle the events in
 * order.
 */
use mio::event::Event;
use mio::net::{UnixListener, UnixStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::VecDeque;
use std::fs;
use std::io::{Error, ErrorKind, Read, Write};
use std::os::unix::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

const SOCKET_PATH_PREPEND: &str = "/tmp/fifo_socket_";
const NUM_EVENTS: usize = 1024;
const POLL_TIMEOUT_MS: u64 = 100;
const BUFFER_SIZE: usize = 1024;

pub struct IpcClientPollHandler {
    pub poll: Poll,
    pub clients: Vec<IpcClient>,
    events: Events,
    roundrobin: VecDeque<Token>,
    event_arr: Vec<Event>,
}
impl IpcClientPollHandler {
    /// Create a new handler for the clients.
    ///
    /// Note that the vector is moved out into the struct,
    /// ideally there should not be a reason to access the
    /// vector manually.
    pub fn new(clients: Vec<IpcClient>) -> Result<IpcClientPollHandler, Error> {
        let mut poll = Poll::new()?;
        let mut token_index = 0; // token index will match the order the array came in
        let mut roundrobin: VecDeque<Token> = VecDeque::new();

        for i in 0..clients.len() {
            roundrobin.push_back(Token(i));
        }

        let mut client_handler = IpcClientPollHandler {
            poll: poll,
            clients: clients,
            events: Events::with_capacity(NUM_EVENTS),
            roundrobin: roundrobin,
            event_arr: vec![],
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

    /// polls clients for data writes to a buffer and returns the number of bytes read
    /// as well as the socket path. Disconnects and reconnects clients accordingly.
    pub fn poll_for_data(&mut self, buf: &mut [u8]) -> Result<(usize, String), Error> {
        // Consider changing the timeout possibly to `None`
        self.poll.poll(
            &mut self.events,
            Some(Duration::from_millis(POLL_TIMEOUT_MS)),
        )?;
        let _: Vec<_> = self
            .events
            .iter()
            .map(|e| self.event_arr.push(e.clone()))
            .collect();

        // TODO: check if this causes issues when adding/removing from roundrobin
        for _ in 0..self.roundrobin.len() {
            let Token(index) = match self.roundrobin.pop_front() {
                Some(t) => t,
                None => break, // means the queue is empty, technically unreachable
            };

            let rem_index = match self
                .event_arr
                .iter()
                .enumerate()
                .find(|(_, x)| x.token() == Token(index))
            {
                Some((rem_index, _)) => rem_index,
                None => {
                    self.roundrobin.push_back(Token(index));
                    continue;
                } // no event corresponding to this index
            };

            let event = self.event_arr.remove(rem_index);
            let client = &mut self.clients[index];

            if event.is_writable() {
                // we can probably read/write from the stream now if it wasn't a spurious event
                client.connected = true;
                println!("Client connected to server on {}", client.socket_path);
            }
            if event.is_read_closed() {
                println!(
                    "Client disconnected from {}, flushing buffer",
                    client.socket_path
                );
                client.connected = false;
            }
            if event.is_writable() {
                self.event_arr.push(event.clone()); // Not the job of this function to write
            }
            if event.is_error() {
                println!("POLL_FOR_DATA IS ERRORFUL");
            }
            if event.is_readable() {
                match client.stream.read(buf) {
                    Ok(0) => {
                        println!("Client disconnected from {}", client.socket_path);
                        client.connected = false;
                    }
                    Ok(bytes_read) => {
                        println!("Read bytes from {}", client.socket_path);
                        self.roundrobin.push_back(Token(index));
                        return Ok((bytes_read, client.socket_path.clone()));
                    }
                    Err(e) => {
                        eprintln!("Could not read from {}\n{}", client.socket_path, e);
                    }
                };
            }
            self.roundrobin.push_back(Token(index));
        }

        // No data was read
        Ok((0, "".to_string()))
    }

    pub fn write_to(&mut self, socket_path: String, buf: &[u8]) -> Result<usize, Error> {
        self.poll.poll(
            &mut self.events,
            Some(Duration::from_millis(POLL_TIMEOUT_MS)),
        )?;

        let _: Vec<_> = self
            .events
            .iter()
            .map(|e| self.event_arr.push(e.clone()))
            .collect();

        let (index, write_client): (usize, &mut IpcClient) = match self
            .clients
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.socket_path == format!("{}{}", SOCKET_PATH_PREPEND, socket_path))
        {
            Some((i, s)) => (i, s),
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Socket path, {}, not found", socket_path),
                ))
            }
        };

        let (rem_index, is_readable) = match self
            .event_arr
            .iter()
            .enumerate()
            .find(|(_, e)| e.token() == Token(index) && e.is_writable())
        {
            Some((i, e)) => (i, e.is_readable()),
            None => {
                return Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    format!("Socket, {}, not writeable", socket_path),
                ))
            }
        };

        if !is_readable {
            self.event_arr.remove(rem_index);
        }

        println!("Writing data to {}", write_client.socket_path);
        return write_client.stream.write(buf);
    }
}

pub struct IpcServerPollHandler {
    pub poll: Poll,
    pub servers: Vec<IpcServer>,
    events: Events,
    roundrobin: VecDeque<Token>,
    event_arr: Vec<Event>,
}
impl IpcServerPollHandler {
    /// Create a new handler for the servers.
    ///
    /// Note that the vector is moved out into the struct,
    /// ideally there should not be a reason to access the
    /// vector manually.
    pub fn new(servers: Vec<IpcServer>) -> Result<IpcServerPollHandler, Error> {
        let mut poll = Poll::new()?;
        let mut token_index = 0; // token index will be 2x the index of the array it came in to keep storage for streams
        let mut roundrobin: VecDeque<Token> = VecDeque::new();

        for i in 0..servers.len() {
            roundrobin.push_back(Token(i * 2));
        }

        let mut server_handler = IpcServerPollHandler {
            poll: poll,
            servers: servers,
            events: Events::with_capacity(NUM_EVENTS),
            roundrobin: roundrobin,
            event_arr: vec![],
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

    /// polls clients for data writes to a buffer and returns the number of bytes read
    /// as well as the socket path. Disconnects and reconnects clients accordingly.
    pub fn poll_for_data(&mut self, buf: &mut [u8]) -> Result<(usize, String), Error> {
        // Consider changing the timeout possibly to `None`
        self.poll.poll(
            &mut self.events,
            Some(Duration::from_millis(POLL_TIMEOUT_MS)),
        )?;

        let _: Vec<_> = self
            .events
            .iter()
            .map(|e| self.event_arr.push(e.clone()))
            .collect();

        // TODO: check if this causes issues when adding/removing from roundrobin
        for _ in 0..self.roundrobin.len() {
            let Token(index) = match self.roundrobin.pop_front() {
                Some(t) => t,
                None => break, // means the queue is empty, technically unreachable
            };

            let rem_index = match self
                .event_arr
                .iter()
                .enumerate()
                .find(|(_, x)| x.token() == Token(index))
            {
                Some((rem_index, _)) => rem_index,
                None => {
                    self.roundrobin.push_back(Token(index));
                    continue;
                } // no event corresponding to this index
            };

            let event = self.event_arr.remove(rem_index);
            let server = &mut self.servers[index / 2];
            let mut remove_from_queue = false;

            match index % 2 {
                0 => {
                    // UnixListener
                    if event.is_readable() {
                        self.accept(index)?;
                    }
                }
                1 => {
                    //UnixStream
                    if event.is_read_closed() {
                        println!("Client has disconnected from {}, flushing buffer prior to deregistering", server.socket_path);
                        server.connected = false;
                        remove_from_queue = true;
                    }
                    if event.is_writable() {
                        self.event_arr.push(event.clone()); // Not the job of this function to write
                    }
                    if event.is_error() {
                        println!("POLL_FOR_DATA IS ERRORFUL");
                    }
                    if event.is_readable() {
                        if let Some(stream) = &mut server.stream {
                            match stream.read(buf) {
                                Ok(0) => {
                                    println!("Client disconnected from {}", server.socket_path);
                                    self.poll.registry().deregister(stream)?;
                                    server.stream = None;
                                    server.connected = false;

                                    return Ok((0, server.socket_path.clone()));
                                }
                                Ok(bytes_read) => {
                                    println!("Recv data from client on {}", server.socket_path);
                                    if !server.connected {
                                        println!("Client disconnected from {}", server.socket_path);
                                        self.poll.registry().deregister(stream)?;
                                        server.stream = None;
                                    }
                                    if !remove_from_queue {
                                        self.roundrobin.push_back(Token(index));
                                    }

                                    return Ok((bytes_read, server.socket_path.clone()));
                                }
                                Err(e) => {
                                    eprintln!("Could not read from {}\n{}", server.socket_path, e);
                                }
                            };
                        }
                    }
                }
                _ => {}
            }

            if !remove_from_queue {
                self.roundrobin.push_back(Token(index));
            }
        }

        // No data was read
        Ok((0, "".to_string()))
    }

    pub fn write_to(&mut self, socket_path: String, buf: &[u8]) -> Result<usize, Error> {
        self.poll.poll(
            &mut self.events,
            Some(Duration::from_millis(POLL_TIMEOUT_MS)),
        )?;

        let _: Vec<_> = self
            .events
            .iter()
            .map(|e| self.event_arr.push(e.clone()))
            .collect();

        let (index, write_server): (usize, &mut IpcServer) = match self
            .servers
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.socket_path == format!("{}{}", SOCKET_PATH_PREPEND, socket_path))
        {
            Some((i, s)) => (i, s),
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Socket path, {}, not found", socket_path),
                ))
            }
        };

        let (rem_index, is_readable) = match self
            .event_arr
            .iter()
            .enumerate()
            .find(|(_, e)| e.token() == Token(2 * index + 1) && e.is_writable())
        {
            Some((i, e)) => (i, e.is_readable()),
            None => {
                return Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    format!("Socket, {}, not writeable", socket_path),
                ))
            }
        };

        if !is_readable {
            self.event_arr.remove(rem_index);
        }

        if let Some(stream) = &mut write_server.stream {
            println!("Writing data to {}", write_server.socket_path);
            return stream.write(buf);
        }

        return Ok(0);
    }

    fn accept(&mut self, index: usize) -> Result<(), Error> {
        let server = &mut self.servers[index / 2];

        match server.listener.accept() {
            Ok((stream, _addr)) => {
                println!("New client connected to {:?}", server.socket_path);
                server.stream = Some(stream);
                server.connected = true;
                self.roundrobin.push_back(Token(index + 1));

                self.poll.registry().register(
                    match &mut server.stream {
                        Some(stream) => stream,
                        None => {
                            panic!("Shouldn't be possible to get here");
                        }
                    },
                    Token(index + 1),
                    Interest::READABLE | Interest::WRITABLE,
                )?;
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }

        Ok(())
    }
}
pub struct IpcClient {
    pub socket_path: String,
    pub stream: UnixStream,
    connected: bool,
}
impl IpcClient {
    pub fn new(socket_name: String) -> Result<IpcClient, Error> {
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
    pub fn new(socket_name: String) -> Result<IpcServer, Error> {
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
