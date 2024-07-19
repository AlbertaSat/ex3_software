/*
Written by Rowan Rasmusson and Devin Headrick
Summer 2024

Modified the IPCInterface methods to use their references to self - rather than return values - to mutate member variables 
    - makes constructor cleaner and reduces code

The difference between the IPC server and client - is that the IPC server has two fds
    - One returned by the socket when its created awaiting a connection request from a client
        - (this will set the POLLIN revent flag but 'accept' instead of 'read' mustbe called to form a connection with the client )

    - Another fd is returned from the accept function, which is the fd that the server will use to communicate data with the client.

    If the server is not connected it will poll the 'conn_fd'
        - otherwise it will just poll the member interface fd (which is the fd returned from the accept function - and changes for each re-connect)

    //TODO - move all server related stuff to the IPC server class -this should leave the IPCInterface unchanged from before this branch and effectively used directly as a client

*/
use nix::libc;
use nix::sys::socket::{self, accept, bind, listen, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::{read, write};
use std::ffi::CString;
use std::io::Error as IoError;
use std::path::Path;
use std::process;
use std::{fs, io};

pub const SOCKET_PATH_PREPEND: &str = "/tmp/fifo_socket_";
pub const IPC_BUFFER_SIZE: usize = 500;
pub const CLIENT_POLL_TIMEOUT_MS: i32 = 100;

#[derive(Clone)]
pub struct IPCServerInterface {
    pub interface: IPCInterface,
    pub conn_fd: Option<i32>, // fd returned by initial socket creation - used to poll for incomming connection req - NOT data
}

impl IPCServerInterface {
    pub fn new_server(socket_name: String) -> Result<IPCServerInterface, std::io::Error> {
        let mut ipc: IPCInterface = IPCInterface {
            fd: 0,
            socket_name: "string".to_string(),
            connected: false,
        };
        ipc.create_socket()?;
        let mut ipc_server = IPCServerInterface {
            interface: ipc.clone(),
            conn_fd: Some(ipc.fd),
        };
        ipc_server.bind_and_listen(socket_name.clone())?;
        Ok(ipc_server)
    }

    /// Bind and listen for incoming connections
    fn bind_and_listen(
        &mut self,
        socket_name: String,
    ) -> Result<(), std::io::Error> {
        let fifo_name = format!("{}{}", SOCKET_PATH_PREPEND, socket_name);
        let socket_path = CString::new(fifo_name).unwrap();
        let path = Path::new(socket_path.to_str().unwrap());
        // Check if the socket file already exists and remove it
        if path.exists() {
            fs::remove_file(path)?;
        }
        let addr = UnixAddr::new(Path::new(socket_path.to_str().unwrap())).unwrap_or_else(|err| {
            eprintln!("Failed to create UnixAddr: {}", err);
            process::exit(1);
        });

        bind(self.interface.fd, &addr).unwrap_or_else(|err| {
            eprintln!("Failed to bind to socket: {}", err);
            process::exit(1);
        });

        listen(self.interface.fd, 10).unwrap_or_else(|err| {
            eprintln!("Failed to listen on socket: {}", err);
            process::exit(1);
        });

        println!("Server listening on {}", socket_path.to_str().unwrap());
        Ok(())
    }

    //  AFTER the client connection is accepted, update the connected flag and change the fd to the one returend by the accepted connection
    pub fn accept_connection(&mut self) -> Result<i32, std::io::Error> {
        let data_fd = accept(self.conn_fd.unwrap()).unwrap_or_else(|err| {
            eprintln!("Failed to accept connection: {}", err);
            process::exit(1);
        });
        self.interface.connected = true;
        self.interface.fd = data_fd;
        Ok(data_fd)
    }
}

#[derive(Clone)]
pub struct IPCInterface {
    pub fd: i32,
    socket_name: String,
    pub connected: bool,
}

impl IPCInterface {
    pub fn new_client(socket_name: String) -> Result<IPCInterface, std::io::Error> {
        let mut ipc: IPCInterface = IPCInterface {
            fd: 0,
            socket_name: "string".to_string(),
            connected: false,
        };
        ipc.create_socket()?;
        ipc.make_connection(socket_name)?;
        Ok(ipc)
    }

    /// create a socket of type SOCK_SEQPACKET to allow passing of information through processes
    fn create_socket(&mut self) -> io::Result<()> {
        let socket_fd = socket::socket(
            AddressFamily::Unix,
            SockType::SeqPacket,
            SockFlag::empty(),
            None,
        )?;
        self.fd = socket_fd;
        Ok(())
    }

    /// Connect client process. True if connection is established.
    fn make_connection(
        &mut self,
        client_name: String,
    ) -> Result<bool, std::io::Error> {
        let fifo_name = format!("{}{}", SOCKET_PATH_PREPEND, client_name);
        let socket_path = CString::new(fifo_name).unwrap();
        let addr = UnixAddr::new(Path::new(socket_path.to_str().unwrap())).unwrap_or_else(|err| {
            eprintln!("Failed to create UnixAddr: {}", err);
            process::exit(1);
        });
        println!("Attempting to connect to {}", socket_path.to_str().unwrap());

        socket::connect(self.fd, &addr).unwrap_or_else(|err| {
            eprintln!("Failed to connect to server: {}", err);
            process::exit(1);
        });

        println!(
            "Successfully Connected to {}, with fd: {}",
            socket_path.to_str().unwrap(),
            self.fd
        );
        self.connected = true;
        Ok(true)
    }
}

/// read bytes over a UNIX SOCK_SEQPACKET socket from a sender. Takes in the fd location to write to.
/// loop{} over this
/// The user needs to create a buffer to pass to the read function.
pub fn read_socket(read_fd: i32, socket_buf: &mut Vec<u8>) -> Result<usize, IoError> {
    // client name is the name of the handler or thing that the client is trying to connect to (fifo is named with this in path)

    //We assume the fd for stdin is always zero. This is the default for UNIX systems and is unlikely to change.

    let mut poll_fds = [libc::pollfd {
        fd: read_fd,
        events: libc::POLLIN,
        revents: 0,
    }];

    let ready = unsafe {
        libc::poll(
            poll_fds.as_mut_ptr(),
            poll_fds.len() as libc::nfds_t,
            CLIENT_POLL_TIMEOUT_MS,
        )
    };

    if ready == -1 {
        eprintln!("poll error");
        process::exit(1);
    }

    for poll_fd in &poll_fds {
        // println!("poll_fd: {:?}", poll_fd);
        if poll_fd.revents != 0 {
            if poll_fd.revents & libc::POLLIN != 0 {
                if poll_fd.fd == read_fd {
                    match read(read_fd, socket_buf) {
                        Ok(ret) => {
                            if ret == 0 {
                                println!("Connection to server dropped. Exiting...");
                                return Err(IoError::new(
                                    io::ErrorKind::UnexpectedEof,
                                    "Connection closed",
                                ));
                            } else {
                                println!("Received: {:?}", socket_buf);
                                return Ok(ret);
                            }
                        }
                        Err(e) => {
                            eprintln!("read error: {:?}", e);
                            return Err(e.into());
                        }
                    }
                }
            }
        }
    }
    return Ok(0);
}

/// Poll vector of interfaces -
pub fn poll_server_interfaces(
    interfaces_vec: &mut Vec<IPCServerInterface>,
    socket_buf: &mut Vec<u8>,
) {
    let mut poll_fds = Vec::new();
    for interface in interfaces_vec.iter() {
        poll_fds.push(libc::pollfd {
            fd: interface.interface.fd,
            events: libc::POLLIN,
            revents: 0,
        });
    }

    let ready = unsafe {
        libc::poll(
            poll_fds.as_mut_ptr(),
            poll_fds.len() as libc::nfds_t,
            CLIENT_POLL_TIMEOUT_MS,
        )
    };

    if ready == -1 {
        eprintln!("poll error");
        process::exit(1);
    }

    // if connected - read
    // if not connected - accept
    for poll_fd in &poll_fds {
        // println!("poll_fd: {:?}", poll_fd);
        if poll_fd.revents != 0 {
            if poll_fd.revents & libc::POLLIN != 0 {
                for interface in interfaces_vec.iter_mut() {
                    if interface.interface.connected {
                        if poll_fd.fd == interface.interface.fd {
                            match read(interface.interface.fd, socket_buf) {
                                Ok(ret) => {
                                    if ret == 0 {
                                        println!("Connection to server dropped. Setting connected flag to false");
                                        interface.interface.connected = false;
                                    } else {
                                        println!("Received: {:?}", socket_buf);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("read error: {:?}", e);
                                    process::exit(1);
                                }
                            }
                        }
                    } else {
                        interface.accept_connection().unwrap();
                        println!("New client accepted on fd: {}", interface.interface.fd);
                    }
                }
            }
        }
    }
}

/// Function for sending data over a specific socket fd. The data should be a
/// serialized Msg struct as a Vec<u8>
pub fn send_over_socket(write_fd: i32, data: Vec<u8>) -> Result<usize, IoError> {
    Ok(write(write_fd, data.as_slice()).unwrap_or_else(|_| {
        eprintln!("write error");
        process::exit(1);
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dfgm_echo() {
        let interface = IPCInterface::new_client("dfgm_handler".to_string());
        let mut socket_buf = vec![0u8; IPC_BUFFER_SIZE];
        loop {
            let output = read_socket(interface.as_ref().unwrap().fd, &mut socket_buf).unwrap();
            if output > 5 {
                break;
            } else {
                continue;
            }
        }
        println!("Sending: {:?}", socket_buf);
        send_over_socket(interface.unwrap().fd, socket_buf.clone()).unwrap();
    }
}