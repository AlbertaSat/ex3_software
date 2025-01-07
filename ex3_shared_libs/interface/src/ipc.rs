/*
Written by Devin Headrick, Rowan Rassmuson, and Drake Boulianne
Summer 2024

*/
use nix::libc;
use nix::sys::socket::{self, accept, bind, listen, socket, Backlog, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::{read, write};
use std::ffi::CString;
use std::fmt::format;
use std::fs::ReadDir;
use std::os::unix::process;
use std::str::from_utf8;
use nix::errno::Errno;
use std::process::exit;
use std::io::Error as IoError;
use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::path::Path;
use std::{fs, io};

const SOCKET_PATH_PREPEND: &str = "/tmp/fifo_socket_";
const CLIENT_PARTIAL_POSTFIX: &str = "_client_";
const SOCKET_DIR: &str = "/tmp";
pub const IPC_BUFFER_SIZE: usize = 4096;
const POLL_TIMEOUT_MS: i32 = 100;

/// Create a unix domain socket with a type of SOCK_DGRAM.
/// Because both server and client need to create a socket, this is a helper function outside of the structs
fn create_socket() -> Result<OwnedFd, IoError> {
    match socket(AddressFamily::Unix, SockType::Datagram, SockFlag::empty(), None) {
        Ok(fd) => Ok(fd), // unsafe { Ok(OwnedFd::from_raw_fd(fd)) },
        Err(e) => Err(IoError::from_raw_os_error(e as i32)),
    }
}

/// Client struct using a unix domain socket of type SOCKSEQ packet, that connects to a server socket
#[derive(Debug)]
pub struct IpcClient {
    pub socket_path: String,
    pub fd: OwnedFd,
    server_addr: UnixAddr,
    pub buffer: [u8; IPC_BUFFER_SIZE],
}

impl Drop for IpcClient {
    // Delete socket file after the client goes out of scope
    fn drop(&mut self) {
        fs::remove_file(self.socket_path.as_str()).unwrap();
    }
}

/// Helper function for the purpose of deleting socket files after IpcClient and IpcServer go out
/// of scope
/// Search the /tmp directory for all client sockets
fn gen_client_socket_path(server_name: &String) -> Result<String, IoError> {
    let paths = fs::read_dir("/tmp/")?;
    let mut max: u8 = 1;
    for path in paths {
        let curr_path = path.unwrap().path().to_str().unwrap().to_string();
        // Check if the path is a client socket for the server we are trying to generate
        // another unique socket for.
        if curr_path.contains(server_name) && curr_path.contains(CLIENT_PARTIAL_POSTFIX) {
            let parts = curr_path.as_str().split("_");
            let parts: Vec<&str> = parts.collect();

            // get the number at the end of the socket path indicating its unique id
            // This value should never be none
            let client_id = match parts.last().unwrap().parse::<u8>() {
                Ok(id) => id,
                Err(e) => {
                    println!("Error parsing id from client socket path: {e}");
                    continue;
                }
            };
            // If the found client id is greater then the current max then
            // change the max to the client id plus one
            if client_id > max {
                max = client_id + 1;
            }
        }
    }

    // This formatted string gives "/tmp/fifo_socket_<server_name>_client_#"
    // where # is the unique id for the client socket.
    let new_socket_path = format!("{}{}{}{}",
        SOCKET_PATH_PREPEND,
        server_name,
        CLIENT_PARTIAL_POSTFIX,
        max
    );
    Ok(new_socket_path)
}


impl IpcClient {
    pub fn new(server_name: String) -> Result<IpcClient, IoError> {
        // creates /tmp/fifo_socket_<server_name>
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, server_name);
        // creates /tmp/fifo_socket_<server_name>_client_#
        // where # is 1-9, this means as of now we can only have 9 sockets speaking to any one
        // server. This is absolute shit
        let socket_path = gen_client_socket_path(&socket_path).unwrap();
        let socket_path_c_str = CString::new(socket_path.clone()).unwrap();
        let socket_addr = UnixAddr::new(socket_path_c_str.as_bytes()).unwrap_or_else(|err|
            {
                eprintln!("Failed to create UnixAddr: {}", err);
                exit(1)

            }
        );

        let socket_fd = create_socket()?;
        match bind(socket_fd.as_raw_fd(), &socket_addr) {
            Ok(()) => {
                println!("Successfully bound client to address {}", &socket_path);
            }
            Err(e) => {
                println!("Failed to bind client socket to address {}", &socket_path);
                exit(1)
            }

        }
        let mut client = IpcClient {
            socket_path: socket_path.clone(),
            fd: socket_fd,
            buffer: [0u8; IPC_BUFFER_SIZE],
        };
        Ok(client)
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

pub fn poll_ipc_clients(clients: &mut Vec<&mut Option<IpcClient>>) -> Result<(usize, String), std::io::Error> {
    //Create poll fd instances for each client
    let mut poll_fds: Vec<libc::pollfd> = Vec::new();
    for client in &mut *clients {
        // Poll data_fd for incoming data
        if client.is_none() {
            return Ok((0,"".to_string()));
        }
        poll_fds.push(libc::pollfd {
            fd: client.as_ref().unwrap().fd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        });
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
        exit(1);
    }
    Ok((0,"".to_string()))
}

pub struct IpcServer {
    pub socket_path: String,
    pub fd: OwnedFd,
    client_addr: Option<UnixAddr>,
    pub buffer: [u8; IPC_BUFFER_SIZE],
}

impl IpcServer {
    pub fn new(socket_name: String) -> Result<IpcServer, IoError> {
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, socket_name);
        let socket_conn_fd = create_socket()?;
        let mut server = IpcServer {
            socket_path,
            fd: socket_conn_fd,
            client_addr: None,
            // data fd is none to start because nothing is connected
            buffer: [0u8; IPC_BUFFER_SIZE],
        };
        server.bind_socket()?;
        // Normally a server would accept conn here - but instead we do this in the polling loop
        Ok(server)
    }

    fn bind_socket(&mut self) -> Result<(), IoError> {
        // Create Unix Address using the string provided by user.
        let socket_path_c_str = CString::new(self.socket_path.clone()).unwrap();
        let addr = UnixAddr::new(socket_path_c_str.as_bytes()).unwrap_or_else(|err| {
            eprintln!("Failed to create UnixAddr: {}", err);
            exit(1)
        });
        // If the path exists already then we remove it (Otherwise we get that the socket is in use
        // on the bind call)
        let path = Path::new(socket_path_c_str.to_str().unwrap());
        if path.exists() {
            fs::remove_file(path)?;
        }

        bind(self.conn_fd.as_raw_fd(), &addr).unwrap_or_else(|err| {
            eprintln!("Failed to bind socket to path: {}", err);
            exit(1)
        });

        println!("Server socket bound to: {}", self.socket_path);

        Ok(())
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
///
/// This function no longer needs to poll for connections. Only input.
pub fn poll_ipc_server_sockets(servers: &mut [&mut Option<IpcServer>]) {
    let mut poll_fds: Vec<libc::pollfd> = Vec::new();

    // Add poll descriptors based on the server's connection state
    for server in servers.iter_mut() {
        // Handle case where server is None
        if server.is_none() {
            return;
        } else if !server.as_ref().unwrap().connected {
            // Poll conn_fd for incoming connections
            poll_fds.push(libc::pollfd {
                fd: server.as_ref().unwrap().conn_fd.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            });
        } else if let Some(ref data_fd) = server.as_ref().unwrap().data_fd {
            // Poll data_fd for incoming data
            poll_fds.push(libc::pollfd {
                fd: data_fd.as_raw_fd(),
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
        exit(1);
    }

    for poll_fd in poll_fds.iter() {
        if poll_fd.revents & libc::POLLIN != 0 {
            let server = servers
                .iter_mut()
                .find(|s| s.as_ref().unwrap().conn_fd.as_raw_fd() == poll_fd.fd ||
                s.as_ref().unwrap().data_fd
                .as_ref().unwrap().as_raw_fd() == poll_fd.fd);
            if let Some(server) = server {
                if !server.as_ref().unwrap().connected {
                    // Handle new connection request from a currently unconnected client
                    server.as_mut().unwrap().accept_connection().unwrap();
                } else if let Some(data_fd) = &server.as_ref().unwrap().data_fd {
                    // Handle incoming data from a connected client
                    let bytes_read = read(data_fd.as_raw_fd(), &mut server.as_mut().unwrap().buffer).unwrap();
                    if bytes_read == 0 {
                        // If 0 bytes read, then the client has disconnected
                        server.as_mut().unwrap().client_disconnected();
                    }
                }
            }
        }
    }
}

/// Wrapper for the unistd lib write fxn
pub fn ipc_write(fd: &OwnedFd, data: &[u8]) -> Result<usize, std::io::Error> {
    match write(
        fd.as_fd(),
        data,
    ) {
        Ok(bytes_read) => Ok(bytes_read),
        Err(e) => {
            eprintln!("Error writing to socket: {}", e);
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
}
