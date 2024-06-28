/*
Written by Devin Headrick
Summer 2024
*/

use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use nix::libc;
use nix::sys::socket::{self, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::{read, write};
use std::ffi::CString;
use std::io;
use std::path::Path;
use std::process;
use message_structure::*;

pub const SOCKET_PATH_PREPEND: &str = "/tmp/fifo_socket_";
pub const IPC_BUFFER_SIZE: usize = 1024;
pub const CLIENT_POLL_TIMEOUT_MS: i32 = 100;
pub const TCP_BUFFER_SIZE: usize = 1024;

/// Interface trait to be implemented by all external interfaces
pub trait Interface {
    /// Send byte data to the interface as a shared slice type byte. Return number of bytes sent
    fn send(&mut self, data: &[u8]) -> Result<usize, Error>;
    /// Read byte data from the interface into a byte slice buffer. Return number of bytes read
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error>;
}

/// TCP Interface for communication with simulated external peripherals
#[derive(Clone)]
pub struct TcpInterface {
    stream: Arc<Mutex<TcpStream>>,
}

impl TcpInterface {
    pub fn new_client(ip: String, port: u16) -> Result<TcpInterface, Error> {
        let stream = TcpStream::connect(format!("{}:{}", ip, port))?;
        stream.try_clone()?.flush()?;
        Ok(TcpInterface {
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    pub fn new_server(ip: String, port: u16) -> Result<TcpInterface, Error> {
        //Create a listener that binds to a socket address (ip:port) and listens for incoming TCP connections
        let listener = std::net::TcpListener::bind(format!("{}:{}", ip, port))?;

        // Accept a new incoming connection on the listener
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    return Ok(TcpInterface {
                        stream: Arc::new(Mutex::new(stream)),
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(e);
                }
            }
        }
        Err(Error::new(std::io::ErrorKind::Other, "No incoming connections"))
    }
}

impl Interface for TcpInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize, Error> {
        let mut stream = self.stream.lock().unwrap();
        let n = stream.write(data)?;
        stream.flush()?;
        Ok(n)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let mut stream = self.stream.lock().unwrap();
        let n = stream.read(buffer)?;
        Ok(n)
    }
}

pub struct IPCInterface {
    fd: i32,
    socket_name: String,
    connected: bool,
}


impl IPCInterface {

    pub fn new(&mut self, socket_name: String) -> IPCInterface {
        let fd = self.create_socket().unwrap();
        let connected = if self.make_connection(fd, socket_name.clone()) {true} else {false};
        IPCInterface {
            fd,
            socket_name,
            connected,
        }
    }

    /// create a socket of type SOCK_SEQPACKET to allow passing of information through processes
    fn create_socket(&mut self) -> io::Result<i32> {
        let socket_fd = socket::socket(
            AddressFamily::Unix,
            SockType::SeqPacket,
            SockFlag::empty(),
            None,
        )?;
        Ok(socket_fd)
    }

    /// Connect client process. True if connection is established.
    fn make_connection(&mut self, socket_fd: i32, client_name: String) -> bool {
        let fifo_name = format!("{}{}", SOCKET_PATH_PREPEND, client_name);
        let socket_path = CString::new(fifo_name).unwrap();
        let addr = UnixAddr::new(Path::new(socket_path.to_str().unwrap())).unwrap_or_else(|err| {
            eprintln!("Failed to create UnixAddr: {}", err);
            process::exit(1);
        });
        println!("Attempting to connect to {}", socket_path.to_str().unwrap());

        socket::connect(socket_fd, &addr).unwrap_or_else(|err| {
            eprintln!("Failed to connect to server: {}", err);
            process::exit(1);
        });

        println!(
            "Successfully Connected to {}, with fd: {}",
            socket_path.to_str().unwrap(),
            socket_fd
        );
        true
    }
}


/// read bytes over a UNIX SOCK_SEQPACKET socket from a sender. Takes in the fd location to write to.
/// loop{} over this 
pub fn read_socket(read_fd: i32) -> usize {
    // client name is the name of the handler or thing that the client is trying to connect to (fifo is named with this in path)

    //We assume the fd for stdin is always zero. This is the default for UNIX systems and is unlikely to change.

    let mut poll_fds = [
        libc::pollfd {
            fd: read_fd,
            events: libc::POLLIN,
            revents: 0,
        },
    ];

    
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
                        let mut socket_buf = vec![0u8; IPC_BUFFER_SIZE];
                        let ret = read(read_fd, &mut socket_buf).unwrap();

                        if ret == 0 {
                            println!("Connection to server dropped. Exiting...");
                            process::exit(0);
                        } else {
                            println!("Received: {}", String::from_utf8_lossy(&socket_buf[..ret]));
                        }
                        return ret;
                    }
                } 
            }
        }
        return 0;
    
}

/// Function for sending data over a specific socket fd. The data should be a 
/// serialized Msg struct as a Vec<u8>
pub fn send_over_socket(sender_fd: i32, data: Vec<u8>) -> usize {
    let mut poll_fds = [
        libc::pollfd {
            fd: sender_fd,
            events: libc::POLLIN,
            revents: 0,
        },
    ];

    loop {
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
                    if poll_fd.fd == sender_fd {
                        // write(sender_fd, data.as_slice()).unwrap_or_else(|_| {
                        //     eprintln!("write error");
                        //     process::exit(1);
                        // });
                        println!("{:?}", data);
                    }
                }  
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_write() {
        let mut ipc: IPCInterface = IPCInterface {
            fd: 0,
            socket_name: "string".to_string(),
            connected: false
            };
        let interface = IPCInterface::new(&mut ipc, "dfgm_handler".to_string());
        // let msg: Msg = Msg::new(0,0,0,0,vec![0,0]);
        // let data: Vec<u8> = serialize_msg(msg).unwrap();
        loop{
        let output = read_socket(interface.fd);
        if output > 5 {
            break;
        } else {
            continue;
        }
        }
    }   
}
