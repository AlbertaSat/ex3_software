/*
Written by Devin Headrick and Rowan Rassmuson
Summer 2024

*/
use nix::libc;
use nix::sys::socket::{self, accept, bind, listen, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::{read, unlink, write};
use std::ffi::CString;
use std::io::Error as IoError;
use std::path::Path;
use std::process;
use std::{fs, io};

pub const SOCKET_PATH_PREPEND: &str = "/tmp/fifo_socket_";
pub const IPC_BUFFER_SIZE: usize = 500;
pub const CLIENT_POLL_TIMEOUT_MS: i32 = 100;

pub enum IpcInterface {
    Server(IpcServer),
    Client(IpcClient),
}

/// Both ipc server and client need to 'create' a socket to connect to using the socket path
impl IpcInterface {
    //Create a unix domain family SOCKSEQPACKET type socket
    fn create_socket() -> Result<(i32), IoError> {
        let socket_fd = socket::socket(
            AddressFamily::Unix,
            SockType::SeqPacket,
            SockFlag::empty(),
            None,
        )?;
        Ok(socket_fd)
    }

    pub fn new_server(socket_name: String) -> IpcInterface {
        IpcInterface::Server(IpcServer::new(socket_name))
    }

    pub fn new_client(socket_name: String) -> IpcInterface {
        IpcInterface::Client(IpcClient::new(socket_name))
    }
}

pub struct IpcClient {
    pub socket_path: String,
    fd: Option<i32>,
    connected: bool,
}
impl IpcClient {
    pub fn new(socket_name: String) -> IpcClient {
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, socket_name);
        // Create socket
        let socket_fd = IpcInterface::create_socket().unwrap();

        //Send connection request to server socket

        // Now you're connected!
        IpcClient {
            socket_path: socket_path,
            fd: None,
            connected: false,
        }
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
}

pub struct IpcServer {
    pub socket_path: String,
    conn_fd: Option<i32>,
    data_fd: Option<i32>,
    connected: bool,
}
impl IpcServer {
    pub fn new(socket_name: String) -> IpcServer {
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, socket_name);

        let socket_conn_fd = IpcInterface::create_socket().unwrap();

        let mut server = IpcServer {
            socket_path: socket_path,
            conn_fd: Some(socket_conn_fd),
            data_fd: None,
            connected: false,
        };

        server.bind_and_listen().unwrap();

        //Regularly would accept conn here - but instead we want to do this in the polling loop
        // server.accept_connection().unwrap();

        server
    }

    fn bind_and_listen(&mut self) -> Result<(), IoError> {
        let socket_path_c_str = CString::new(self.socket_path.clone()).unwrap();
        let addr = UnixAddr::new(socket_path_c_str.as_bytes()).unwrap_or_else(|err| {
            eprintln!("Failed to create UnixAddr: {}", err);
            process::exit(1)
        });

        let path = Path::new(socket_path_c_str.to_str().unwrap());
        // Check if the socket file already exists and remove it
        if path.exists() {
            fs::remove_file(path)?;
        }

        // Bind socket to path
        bind(self.conn_fd.unwrap(), &addr).unwrap_or_else(|err| {
            eprintln!("Failed to bind socket to path: {}", err);
            process::exit(1)
        });

        // Listen to socket
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
        println!("Accepted connection from client socket");
        Ok(())
    }
}

pub fn poll_ipc_servers(mut servers: Vec<&mut IpcServer>) {
    let mut poll_fds: Vec<libc::pollfd> = Vec::new();

    // Add poll descriptors based on the server's connection state
    for server in &mut servers {
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
            CLIENT_POLL_TIMEOUT_MS,
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
                    // Handle new connection
                    server.accept_connection().unwrap();
                } else if let Some(data_fd) = server.data_fd {
                    // Handle incoming data
                    let mut buffer = [0u8; IPC_BUFFER_SIZE];
                    let bytes_read = read(data_fd, &mut buffer).unwrap();
                    if bytes_read == 0 {
                        // Client disconnected
                        server.connected = false;
                        server.data_fd = None;
                        println!("Client disconnected");
                    } else {
                        println!(
                            "Received {} bytes from client: {:?}",
                            bytes_read,
                            String::from_utf8_lossy(&buffer)
                        );
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        
                        // Echo the data back to the client
                        write(data_fd, &buffer).unwrap();
                    }
                }
            }
        }
    }
}

//Supposedly 'outer' enum is just used for matching - after this then access the 'inner' enum variant to access the data
#[cfg(test)]
mod tests {
    use nix::poll;

    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_server_creation_and_listening() {
        let server_name = "test_server".to_string();
        let mut ipc_server = IpcServer::new(server_name.clone());

        while true {
            poll_ipc_servers(vec![&mut ipc_server]);
        }
    }

    #[test]
    /// Run this after running the server creation and listen test above
    fn test_client_connection_to_server() {
        let server_name = "test_server".to_string();
        let server_name_clone = server_name.clone();
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, server_name);

        let mut ipc_client = IpcClient::new(server_name_clone);
        ipc_client.fd = Some(IpcInterface::create_socket().unwrap());

        assert_eq!(ipc_client.socket_path, socket_path);
        assert!(!ipc_client.connected);

        match ipc_client.connect_to_server() {
            Ok(_) => {
                assert!(ipc_client.connected);
                println!("Client connected to server successfully.");
            }
            Err(e) => {
                panic!("Client failed to connect to server: {:?}", e);
            }
        }

        // TODO - replace this with a fxn to write data to a socket
        // Write data to the server now
        let data = "Hello, server!";
        let data_c_str = CString::new(data).unwrap();
        let data_bytes = data_c_str.as_bytes_with_nul();
        let data_fd = ipc_client.fd.unwrap();
        write(data_fd, data_bytes).unwrap();

        //TODO - replace this with a poll loop
        // Read data from the server
        let mut buffer = [0u8; IPC_BUFFER_SIZE];
        let bytes_read = read(data_fd, &mut buffer).unwrap();
        println!(
            "Received {} bytes from server: {:?}",
            bytes_read,
            String::from_utf8_lossy(&buffer)
        );
    }
}
