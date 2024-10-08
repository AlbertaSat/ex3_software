/*
Written by Devin Headrick and Rowan Rassmuson
Summer 2024

*/
use nix::libc;
use nix::sys::socket::{self, accept, bind, listen, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::{read, write};
use std::ffi::CString;
use std::io::Error as IoError;
use std::path::Path;
use std::process;
use std::{fs, io};

const SOCKET_PATH_PREPEND: &str = "/tmp/fifo_socket_";
pub const IPC_BUFFER_SIZE: usize = 4096;
const POLL_TIMEOUT_MS: i32 = 100;

/// Create a unix domain socket with a type of SOCKSEQ packet.
/// Because both server and client need to create a socket, this is a helper function outside of the structs
fn create_socket() -> Result<i32, IoError> {
    let socket_fd = socket::socket(
        AddressFamily::Unix,
        SockType::SeqPacket,
        SockFlag::empty(),
        None,
    )?;
    Ok(socket_fd)
}

/// Client struct using a unix domain socket of type SOCKSEQ packet, that connects to a server socket
#[derive(Clone)]
pub struct IpcClient {
    pub socket_path: String,
    pub fd: Option<i32>,
    connected: bool,
    pub buffer: [u8; IPC_BUFFER_SIZE],
}
impl IpcClient {
    pub fn new(socket_name: String) -> Result<IpcClient, IoError> {
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, socket_name);
        let socket_fd = create_socket()?;
        let mut client = IpcClient {
            socket_path: socket_path.clone(),
            fd: Some(socket_fd),
            connected: false,
            buffer: [0u8; IPC_BUFFER_SIZE],
        };
        client.connect_to_server()?; // Sends connection request to server
        Ok(client)
    }

    fn connect_to_server(&mut self) -> Result<(), IoError> {
        let socket_path_c_str = CString::new(self.socket_path.clone()).unwrap();
        let addr = UnixAddr::new(socket_path_c_str.as_bytes()).unwrap_or_else(|err| {
            eprintln!("Failed to create UnixAddr: {}", err);
            process::exit(1)
        });
        println!(
            "Attempting to connect to server socket at: {}",
            self.socket_path
        );
        socket::connect(self.fd.unwrap(), &addr).unwrap_or_else(|err| {
            eprintln!("Failed to connect to server socket: {}", err);
            process::exit(1)
        });
        println!("Connected to server socket at: {}", self.socket_path);
        self.connected = true;

        Ok(())
    }

    /// Users of this lib can call this to clear the buffer - otherwise the preivous read data will remain
    /// the IPC client has no way of knowing when the user is done with the data in its buffer, so it is the responsibility of the user to clear it
    pub fn clear_buffer(&mut self) {
        self.buffer = [0u8; IPC_BUFFER_SIZE];
        println!("Buffer cleared");
    }

    /// Returns the buffer in its current state for directly reading values in real time.
    /// **This function also clears the buffer after the read!**
    pub fn read_buffer(&mut self) -> Vec<u8> {
        let tmp = self.buffer.to_vec();
        self.clear_buffer();
        tmp
    }
}

pub fn poll_ipc_clients(clients: &mut Vec<&mut IpcClient>) -> Result<(usize, String), std::io::Error> {
    //Create poll fd instances for each client
    let mut poll_fds: Vec<libc::pollfd> = Vec::new();
    for client in &mut *clients {
        if let Some(data_fd) = client.fd {
            // Poll data_fd for incoming data
            poll_fds.push(libc::pollfd {
                fd: data_fd,
                events: libc::POLLIN,
                revents: 0,
            });
        }
    }

    let poll_result = unsafe {
        libc::poll(
            poll_fds.as_mut_ptr(),
            poll_fds.len() as libc::nfds_t,
            POLL_TIMEOUT_MS,
        )
    };

    if poll_result < 0 {
        eprintln!(
            "Error polling for client data: {}",
            io::Error::last_os_error()
        );
        process::exit(1);
    }

    //Poll each client for incoming data
    for poll_fd in poll_fds.iter() {
        if poll_fd.revents & libc::POLLIN != 0 {
            let client = clients.iter_mut().find(|s| s.fd == Some(poll_fd.fd));
            if let Some(client) = client {
                if let Some(data_fd) = client.fd {
                    // Handle incoming data from a connected client
                    let bytes_read = read(data_fd, &mut client.buffer)?;
                    if bytes_read > 0 {
                        println!(
                            "Received {} bytes on socket {}",
                            bytes_read, client.socket_path
                        );
                        return Ok((bytes_read, client.socket_path.clone()));
                    }
                }
            }
        }
    }
    Ok((0,"".to_string()))
}
#[derive(Clone)]
pub struct IpcServer {
    pub socket_path: String,
    conn_fd: Option<i32>,
    pub data_fd: Option<i32>,
    connected: bool,
    pub buffer: [u8; IPC_BUFFER_SIZE],
}
impl IpcServer {
    pub fn new(socket_name: String) -> Result<IpcServer, IoError> {
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, socket_name);
        let socket_conn_fd = create_socket()?;
        let mut server = IpcServer {
            socket_path,
            conn_fd: Some(socket_conn_fd),
            data_fd: None,
            connected: false,
            buffer: [0u8; IPC_BUFFER_SIZE],
        };
        server.bind_and_listen()?;
        // Normally a server would accept conn here - but instead we do this in the polling loop
        Ok(server)
    }

    fn bind_and_listen(&mut self) -> Result<(), IoError> {
        let socket_path_c_str = CString::new(self.socket_path.clone()).unwrap();
        let addr = UnixAddr::new(socket_path_c_str.as_bytes()).unwrap_or_else(|err| {
            eprintln!("Failed to create UnixAddr: {}", err);
            process::exit(1)
        });

        let path = Path::new(socket_path_c_str.to_str().unwrap());
        if path.exists() {
            fs::remove_file(path)?;
        }

        bind(self.conn_fd.unwrap(), &addr).unwrap_or_else(|err| {
            eprintln!("Failed to bind socket to path: {}", err);
            process::exit(1)
        });

        listen(self.conn_fd.unwrap(), 5).unwrap_or_else(|err| {
            eprintln!("Failed to listen to socket: {}", err);
            process::exit(1)
        });

        println!("Listening to socket at: {}", self.socket_path);

        Ok(())
    }

    fn accept_connection(&mut self) -> Result<(), IoError> {
        let fd = accept(self.conn_fd.unwrap()).unwrap_or_else(|err| {
            eprintln!("Failed to accept connection: {}", err);
            process::exit(1)
        });
        self.data_fd = Some(fd);
        self.connected = true;
        println!("Accepted connection from client socket {} on data fd {:?}", self.socket_path, self.data_fd);
        Ok(())
    }

    fn client_disconnected(&mut self) {
        self.connected = false;
        self.data_fd = None;
        println!("Client disconnected");
    }

    /// Users of this lib can call this to clear the buffer - otherwise the preivous read data will remain
    ///  the IPC server has no way of knowing when the user is done with the data in its buffer, so it is the responsibility of the user to clear it
    pub fn clear_buffer(&mut self) {
        self.buffer = [0u8; IPC_BUFFER_SIZE];
        println!("Buffer cleared");
    }

    /// Returns the buffer in its current state for directly reading values in real time.
    /// **This function also clears the buffer after the read!**
    pub fn read_buffer(&mut self) -> Vec<u8> {
        let tmp = self.buffer.to_vec();
        self.clear_buffer();
        tmp
    }
}


/// Takes a vector of mutable referenced IpcServers and polls them for incoming data
/// The IpcServers must be mutable because the connected state and data_fd are mutated in the polling loop
pub fn poll_ipc_server_sockets(servers: &mut [&mut IpcServer]) {
    let mut poll_fds: Vec<libc::pollfd> = Vec::new();

    // Add poll descriptors based on the server's connection state
    for server in servers.iter_mut() {
        if let Some(fd) = server.conn_fd {
            if !server.connected {
                // Poll conn_fd for incoming connections
                poll_fds.push(libc::pollfd {
                    fd,
                    events: libc::POLLIN,
                    revents: 0,
                });
            } else if let Some(data_fd) = server.data_fd {
                // Poll data_fd for incoming data
                poll_fds.push(libc::pollfd {
                    fd: data_fd,
                    events: libc::POLLIN,
                    revents: 0,
                });
            }
        }
    }

    let poll_result = unsafe {
        libc::poll(
            poll_fds.as_mut_ptr(),
            poll_fds.len() as libc::nfds_t,
            POLL_TIMEOUT_MS,
        )
    };

    if poll_result < 0 {
        eprintln!(
            "Error polling for client data: {}",
            io::Error::last_os_error()
        );
        process::exit(1);
    }

    for poll_fd in poll_fds.iter() {
        if poll_fd.revents & libc::POLLIN != 0 {
            let server = servers
                .iter_mut()
                .find(|s| s.conn_fd == Some(poll_fd.fd) || s.data_fd == Some(poll_fd.fd));
            if let Some(server) = server {
                if !server.connected {
                    // Handle new connection request from a currently unconnected client
                    server.accept_connection().unwrap();
                } else if let Some(data_fd) = server.data_fd {
                    // Handle incoming data from a connected client
                    let bytes_read = read(data_fd, &mut server.buffer).unwrap();
                    if bytes_read == 0 {
                        // If 0 bytes read, then the client has disconnected
                        server.client_disconnected();
                    }
                }
            }
        }
    }
}

/// Wrapper for the unistd lib write fxn
pub fn ipc_write(fd: Option<i32>, data: &[u8]) -> Result<usize, std::io::Error> {
    match write(
        fd.expect("Write takes a file descriptor in form i32."),
        data,
    ) {
        Ok(bytes_read) => Ok(bytes_read),
        Err(e) => {
            eprintln!("Error reading from socket: {}", e);
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
}
