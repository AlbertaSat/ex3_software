/*
Written by Devin Headrick and Rowan Rasmusson
Summer 2024
*/
use std::net::{TcpStream, TcpListener};
use std::io::{Error, Read, Write};
use std::os::unix::io::AsRawFd;
use nix::libc;
use std::io;

pub const BUFFER_SIZE: usize = 1024;
const CLIENT_POLL_TIMEOUT_MS: i32 = 100;

/// Interface trait to be implemented by all external interfaces
pub trait Interface {
    /// Send byte data to the interface as a shared slice type byte. Return number of bytes sent
    fn send(&mut self, data: &[u8]) -> Result<usize, Error>;
    /// Read byte data from the interface into a byte slice buffer. Return number of bytes read
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error>;

}

/// TCP Interface for communication with simulated external peripherals
pub struct TcpInterface {
    ip: String,
    port: u16,
    fd: [libc::pollfd; 1],
    stream: TcpStream,
}

impl TcpInterface {
    pub fn new_client(ip: String, port: u16) -> Result<TcpInterface, Error> {
        let stream = TcpStream::connect(format!("{}:{}", ip, port))?;
        let tcp_fd = stream.as_raw_fd();
        let poll_fds = [
            libc::pollfd {
                fd: tcp_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        Ok(TcpInterface {
            ip,
            port,
            fd: poll_fds,
            stream,
        })
    }

    pub fn new_server(ip: String, port: u16) -> Result<TcpInterface, Error> {
        let listener = TcpListener::bind(format!("{}:{}", ip, port))?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let tcp_fd = stream.as_raw_fd();
                    let poll_fds = [
                        libc::pollfd {
                            fd: tcp_fd,
                            events: libc::POLLIN,
                            revents: 0,
                        },
                    ];
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    return Ok(TcpInterface {
                        ip,
                        port,
                        fd: poll_fds,
                        stream,
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(e);
                }
            }
        }
        Err(Error::new(
            io::ErrorKind::Other,
            "No incoming connections",
        ))
    }
}

impl Interface for TcpInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize, Error> {
        let n = self.stream.write(data)?;
        self.stream.flush()?;
        Ok(n)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let ready = unsafe {
            libc::poll(
                self.fd.as_mut_ptr(),
                1 as libc::nfds_t,
                CLIENT_POLL_TIMEOUT_MS,
            )
        };

        if ready == -1 {
            return Err(Error::new(io::ErrorKind::Other, "poll error"));
        }

        if self.fd[0].revents & libc::POLLIN != 0 {
            let n = self.stream.read(buffer)?;
            return Ok(n);
        }

        Err(Error::new(io::ErrorKind::WouldBlock, "No Data Available"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests are meant to be run with a netcat TCP server to 
    // ensure the functionality of read and write

    #[test]
    fn test_handler_write() {
        let mut client_interface = TcpInterface::new_client("127.0.0.1".to_string(), 8080).unwrap();
        if let Ok(n) = TcpInterface::send(&mut client_interface, &[48,48,48,48,48]) {
            println!("Sent {} bytes", n);
        } else {
            // couldn't send bytes
        }
    }
    #[test]
    fn test_handler_read() {
        let mut client_interface = TcpInterface::new_client("127.0.0.1".to_string(), 8080).unwrap();
        let mut buffer = [0u8;BUFFER_SIZE];
        loop {
        if let Ok(n) = TcpInterface::read(&mut client_interface, &mut buffer) {
            println!("got dem bytes: {:?}", buffer);
            if n > 0 {
                break;
            } else {
                continue;
            }
        } else {
            println!("No bytes to read");
        }
        }
    }
}