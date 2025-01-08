/*
Written by Devin Headrick, Rowan Rassmuson, and Drake Boulianne
Summer 2024

*/
use nix::libc;
use nix::sys::socket::{self, bind, socket, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::write;
use std::ffi::CString;
use std::process::exit;
use std::io::Error as IoError;
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::path::Path;
use std::{fs, io};
use super::Interface;

// TODO: Implement drop trait so that IpcClients socket paths are deleted when they go out of scope

const SOCKET_PATH_PREPEND: &str = "/tmp/fifo_socket_";
const CLIENT_PARTIAL_POSTFIX: &str = "_client_";
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
    pub server_addr: Option<UnixAddr>,
    pub buffer: [u8; IPC_BUFFER_SIZE],
}

impl Interface for IpcClient {
    /// reads data from the given server, if client_addr_op is not None then we assign this
    /// client's unix address to the client_addr field in the IpcServer Struct.
    /// *** This function does not read into IpcServer's buffer field ***
    fn read(&mut self, data: &mut [u8]) -> Result<usize, IoError> {
        let (bytes_read, server_addr_op) = socket::recvfrom::<UnixAddr>(self.fd.as_raw_fd(), data)?;
        if server_addr_op.is_some() {
            // In this conditional we can check if we accidentally recv data from socket that is
            // not our server
            self.server_addr = server_addr_op;
        } else {
            return Ok(0)
        }
        Ok(bytes_read)
    }

    /// Sends the data to the client via IpcServer's client_addr field
    /// if client_addr is none, then we return NotFound error
    fn send(&mut self, data: &[u8]) -> Result<usize, IoError> {
        if self.server_addr.is_none() {
            // Return not found error
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("No client address for server {}", self.socket_path)));
        }
        // Server Address should never unwrap here
        let bytes_sent = socket::sendto(self.fd.as_raw_fd(), data, &self.server_addr.unwrap(), socket::MsgFlags::empty())?;
        Ok(bytes_sent)
    }
}

impl Drop for IpcClient {
    // Delete socket file after the client goes out of scope
    // This is not working right now. needs testing.
    fn drop(&mut self) {
        fs::remove_file(self.socket_path.as_str()).unwrap();
    }
}

impl IpcClient {
    pub fn new(server_name: String) -> Result<IpcClient, IoError> {
        // Creates /tmp/fifo_socket_<server_name>_client_#
        let socket_path = gen_client_socket_path(&server_name).unwrap();
        let socket_path_c_str = CString::new(socket_path.clone()).unwrap();
        let socket_addr = UnixAddr::new(socket_path_c_str.as_bytes()).unwrap_or_else(|err|
            {
                eprintln!("Failed to create UnixAddr for client: {}", err);
                exit(1)
            }
        );

        // Create socket and bind previously created Unix Address to socket
        let socket_fd = create_socket()?;
        match bind(socket_fd.as_raw_fd(), &socket_addr) {
            Ok(()) => {
                println!("Successfully bound client to address {}", &socket_path);
            }
            Err(e) => {
                eprintln!("Failed to bind client socket to address {} : {}", &socket_path, e);
                exit(1)
            }
        }

        // Initialize server addr to be /tmp/fifo_socket_<server_name>
        let server_name = format!("{}{}", SOCKET_PATH_PREPEND, server_name);
        let server_address_c_str = CString::new(server_name.clone()).unwrap();
        let server_addr = UnixAddr::new(server_address_c_str.as_bytes()).unwrap_or_else(
            |err| {
                eprintln!("Failed to create UnixAddr for server: {}", err);
                exit(1);
            }
        );
        println!("Successfully set server address to {}", server_name);
        let client = IpcClient {
            socket_path: socket_path.clone(),
            fd: socket_fd,
            server_addr: Some(server_addr),
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
    pub fn send(&mut self, data: &[u8]) -> Result<usize, IoError>{
        if self.server_addr.is_none() {
            eprintln!("No server found for client.");
            // return no such device or address error (ENXIO)
            return Err(io::Error::from_raw_os_error(6));
        }
        // Server Address should never unwrap here
        let ret = socket::sendto(self.fd.as_raw_fd(), data, &self.server_addr.unwrap(), socket::MsgFlags::empty())?;
        Ok(ret)
    }
}

/// This function polls each ipc client in the provided vector of optional clients.
/// Returns the number of bytes read and the optional address which it was sent from if successful,
/// if it fails it returns an IO Error.
pub fn poll_ipc_clients(clients: &mut Vec<&mut Option<IpcClient>>) -> Result<(usize, Option<UnixAddr>), std::io::Error> {
    //Create poll fd instances for each client
    let mut poll_fds: Vec<libc::pollfd> = Vec::new();
    for client in &mut *clients {
        // Poll fd for incoming data
        if client.is_none() {
            return Ok((0, None));
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

    //Poll each client for incoming data
    for poll_fd in poll_fds.iter() {
        if poll_fd.revents & libc::POLLIN != 0 {
            let client = clients.iter_mut().find(|s| s.as_ref().unwrap().fd.as_raw_fd() == poll_fd.fd);
            if let Some(client) = client {
                // Handle incoming data from a client
                let (bytes_read, recv_addr) = socket::recvfrom::<UnixAddr> (client.as_ref().unwrap().fd.as_raw_fd(), &mut client.as_mut().unwrap().buffer)?;
                if bytes_read > 0 {
                    println!(
                        "Received {} bytes on socket {} from {:?}",
                        bytes_read, client.as_ref().unwrap().socket_path, &recv_addr.unwrap().path().unwrap().to_str()
                    );
                    return Ok((bytes_read, recv_addr));
                }
            }
        }
    }
    Ok((0, None))
}

pub struct IpcServer {
    pub socket_path: String,
    pub fd: OwnedFd,
    // client addr is the unix addr of the client most recently talked to by the server
    pub client_addr: Option<UnixAddr>,
    pub buffer: [u8; IPC_BUFFER_SIZE],
}

impl Interface for IpcServer {
    /// reads data from the given server, if client_addr_op is not None then we assign this
    /// client's unix address to the client_addr field in the IpcServer Struct.
    /// *** This function does not read into IpcServer's buffer field ***
    fn read(&mut self, data: &mut [u8]) -> Result<usize, IoError> {
        let (bytes_read, client_addr_op) = socket::recvfrom::<UnixAddr>(self.fd.as_raw_fd(), data)?;
        if client_addr_op.is_some() {
            self.client_addr = client_addr_op;
        } else {
            return Ok(0)
        }
        Ok(bytes_read)
    }

    /// Sends the data to the client via IpcServer's client_addr field
    /// if client_addr is none, then we return NotFound error
    fn send(&mut self, data: &[u8]) -> Result<usize, IoError> {
        if self.client_addr.is_none() {
            // Return not found error
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("No client address for server {}", self.socket_path)));
        }
        // Server Address should never unwrap here
        let bytes_sent = socket::sendto(self.fd.as_raw_fd(), data, &self.client_addr.unwrap(), socket::MsgFlags::empty())?;
        Ok(bytes_sent)
    }
}

impl IpcServer {
    pub fn new(socket_name: String) -> Result<IpcServer, IoError> {
        let socket_path = format!("{}{}", SOCKET_PATH_PREPEND, socket_name);
        // This is a measure for when servers need to initiate communication with the client
        // socket, this should not really happen. If needed we can manually set the unix address
        // in the server struct.
        let client_path = format!("{}{}{}", socket_path, CLIENT_PARTIAL_POSTFIX, 1);
        let client_path_c_str = CString::new(client_path).unwrap();
        let client_unix_addr = UnixAddr::new(client_path_c_str.as_bytes()).unwrap();
        let socket_conn_fd = create_socket()?;
        let mut server = IpcServer {
            socket_path,
            fd: socket_conn_fd,
            // Client socket is none to start
            client_addr: Some(client_unix_addr),
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

        bind(self.fd.as_raw_fd(), &addr).unwrap_or_else(|err| {
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
pub fn poll_ipc_server_sockets(servers: &mut Vec<&mut Option<IpcServer>>) -> Result<(usize, Option<UnixAddr>), IoError> {
    //Create poll fd instances for each client
    let mut poll_fds: Vec<libc::pollfd> = Vec::new();
    for server in &mut *servers {
        // Poll fd for incoming data
        if server.is_none() {
            return Ok((0, None));
        }
        poll_fds.push(libc::pollfd {
            fd: server.as_ref().unwrap().fd.as_raw_fd(),
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

    //Poll each client for incoming data
    for poll_fd in poll_fds.iter() {
        if poll_fd.revents & libc::POLLIN != 0 {
            let mut server = servers.iter_mut().find(|s| s.as_ref().unwrap().fd.as_raw_fd() == poll_fd.fd);
            if let Some(ref mut server) = server {
                // Handle incoming data from a client
                let (bytes_read, recv_addr) = socket::recvfrom::<UnixAddr>(server.as_ref().unwrap().fd.as_raw_fd(), &mut server.as_mut().unwrap().buffer)?;
                if bytes_read > 0 {
                    println!(
                        "Received {} bytes on socket {} from {:?}",
                        bytes_read, server.as_ref().unwrap().socket_path, &recv_addr
                    );
                    // Set the servers client address so we know which client to respond to.
                    server.as_mut().unwrap().client_addr = recv_addr;
                    return Ok((bytes_read, recv_addr));
                }
            }
        }
    }
    Ok((0, None))
}

/// Wrapper for the unistd lib write fxn
/// Deprecated
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

/// Function to generate unique client address, currently the maximum number of
/// clients per server is bottlenecked to 255 (unique identifier is u8)
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::Duration;
    use std::thread::sleep;
    #[test]
    fn test_gen_client_path() {
        let _ = File::create("/tmp/fifo_socket_TEST_client_20").unwrap();
        let socket_client_21 = gen_client_socket_path(&String::from("TEST")).unwrap();
        // Create a string that should be the same as the path string created by
        // gen_client_socket_path function.
        let dummy_client_21_string = String::from("/tmp/fifo_socket_TEST_client_21");
        assert!(socket_client_21 == dummy_client_21_string);
        // Clean up after testing is done
        fs::remove_file("/tmp/fifo_socket_TEST_client_20").unwrap();

    }

    #[test]
    fn test_client_socket_drop() {
        {
            let _ = IpcClient::new("TEST".to_string());
            // test to ensure file is created
        }
        assert!(!(fs::exists("/tmp/fifo_socket_TEST_client_1").unwrap()));
    }

    #[test]
    fn test_server_switching_client_addr() {
        let mut server = IpcServer::new("TEST".to_string()).unwrap();
        let mut client_1 = IpcClient::new("TEST".to_string()).unwrap();
        let mut client_2 = IpcClient::new("TEST".to_string()).unwrap();

        client_1.send("dummy_data".as_bytes()).unwrap();
        // Sleep a small amount of time for server to recv data
        sleep(Duration::from_millis(5));
        let mut buf = [0; 50];
        server.read(&mut buf).unwrap();
        // Check to see that client_addr has changed successfully to client_1's socket_path
        println!("client_addr: {} | socket_path: {}", 
            server.client_addr.unwrap().path().unwrap().to_str().unwrap(),
            client_1.socket_path
        );
        assert_eq!(server.client_addr.unwrap().path().unwrap().to_str().unwrap(), client_1.socket_path);
        client_2.send("dummy_data2".as_bytes()).unwrap();
        sleep(Duration::from_millis(5));
        server.read(&mut buf).unwrap();
        // Check to see that client_addr has changed successfully to client_2's socket path
        println!("client_addr: {} | socket_path: {}", 
            server.client_addr.unwrap().path().unwrap().to_str().unwrap(),
            client_2.socket_path
        );
        assert_eq!(server.client_addr.unwrap().path().unwrap().to_str().unwrap(), client_2.socket_path);
    }
}
