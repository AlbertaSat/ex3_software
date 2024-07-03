/*
Written by Devin Headrick and Rowan Rasmusson
Summer 2024
*/
use std::net::TcpStream;
use std::io::{Error, Read, Write};
use std::sync::{Arc, Mutex};
use std::os::unix::io::AsRawFd;
use nix::{libc, poll};

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
#[derive(Clone)]
pub struct TcpInterface {
    ip: String,
    port: u16,
    fd: [libc::pollfd; 1],
}

impl TcpInterface {
    pub fn new_client(ip: String, port: u16) -> Result<TcpInterface, Error> {
        let stream = TcpStream::connect(format!("{}:{}", ip, port))?;
        stream.try_clone()?.flush()?;
        let tcp_fd: i32 = stream.as_raw_fd();
        let mut poll_fds = [
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
        })
    }

    pub fn new_server(ip: String, port: u16) -> Result<TcpInterface, Error> {
        //Create a listener that binds to a socket address (ip:port) and listens for incoming TCP connections
        let listener = std::net::TcpListener::bind(format!("{}:{}", ip, port))?;

        // Accept a new incoming connection on the listener
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let tcp_fd: i32 = stream.as_raw_fd();
                    let mut poll_fds = [
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
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(e);
                }
            }
        }
        Err(Error::new(
            std::io::ErrorKind::Other,
            "No incoming connections",
        ))
    }
}

impl Interface for TcpInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize, Error> {
        let stream = TcpStream::connect(format!("{}:{}", self.ip, self. port));
        let mut stream = stream.unwrap();
        let n = stream.write(data)?;
        stream.flush()?;
        Ok(n)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let stream = TcpStream::connect(format!("{}:{}", self.ip, self. port));
        let mut stream = stream.unwrap();

        let ready = unsafe {
            libc::poll(
                self.fd.as_mut_ptr(),
                1 as libc::nfds_t,
                CLIENT_POLL_TIMEOUT_MS,
            )
        };

        if ready == -1 {
            return Err(Error::new(std::io::ErrorKind::Other, "poll error"));
        }

        if self.fd[0].revents & libc::POLLIN != 0 {
            let n = stream.read(buffer)?;
            return Ok(n);
        }

        Err(Error::new(std::io::ErrorKind::WouldBlock, "No Data Available"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use rand::Rng;

    fn setup_server_client() -> (TcpInterface, TcpInterface) {
        let server_ip = "127.0.0.1";
        let server_port = rand::thread_rng().gen_range(8000..u16::MAX); // is this ok? for testing?

        let server_thread = thread::spawn(move || {
            TcpInterface::new_server(server_ip.to_string(), server_port).unwrap()
        });

        // Give the server some time to start
        thread::sleep(Duration::from_millis(100));

        let client = TcpInterface::new_client(server_ip.to_string(), server_port).unwrap();
        let server = server_thread.join().unwrap();

        (server, client)
    }

    // --------- Unit Tests ----------

    #[test]
    fn test_polling_read() {
        let (mut server, mut client) = setup_server_client();

        let client_thread = thread::spawn(move || {
            client.send(b"Hello, world!").unwrap();
        });

        let mut buffer = vec![0; BUFFER_SIZE];
        let n = server.read(&mut buffer).unwrap();

        assert_eq!(n, 13);
        assert_eq!(&buffer[..n], b"Hello, world!");

        client_thread.join().unwrap();
    }

    #[test]
    fn test_polling_no_data() {
        let (mut server, _client) = setup_server_client();

        let mut buffer = vec![0; BUFFER_SIZE];
        match server.read(&mut buffer) {
            Ok(_) => panic!("Expected WouldBlock error"),
            Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::WouldBlock),
        }
    }

}