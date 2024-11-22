/*
Written by Devin Headrick and Rowan Rasmusson
Summer 2024
*/
use super::Interface;
use nix::libc;
use std::io;
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;

pub const BUFFER_SIZE: usize = 1024;
const CLIENT_POLL_TIMEOUT_MS: i32 = 100;

/// TCP Interface for communication with simulated external peripherals
#[allow(dead_code)]
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
        let poll_fds = [libc::pollfd {
            fd: tcp_fd,
            events: libc::POLLIN,
            revents: 0,
        }];
        Ok(TcpInterface {
            ip,
            port,
            fd: poll_fds,
            stream,
        })
    }

    pub fn new_server(ip: String, port: u16) -> Result<TcpInterface, Error> {
        let listener = TcpListener::bind(format!("{}:{}", ip, port))?;
        if let Some(stream) = listener.incoming().next() {
            match stream {
                Ok(stream) => {
                    let tcp_fd = stream.as_raw_fd();
                    let poll_fds = [libc::pollfd {
                        fd: tcp_fd,
                        events: libc::POLLIN,
                        revents: 0,
                    }];
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
        Err(Error::new(io::ErrorKind::Other, "No incoming connections"))
    }
}

impl Interface for TcpInterface {
    fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
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

        if self.fd[0].revents != 0 {
            if self.fd[0].revents & libc::POLLIN != 0 {
                let n = self.stream.read(buffer)?;
                return Ok(n);
            } else if self.fd[0].revents & libc::POLLHUP != 0
                || self.fd[0].revents & libc::POLLERR != 0
            {
                return Err(Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Connection Closed",
                ));
            }
        }
        Ok(0)
        //Err(Error::new(io::ErrorKind::WouldBlock, "No Data Available"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;
    use std::{
        process::{Command, Stdio},
        str, thread,
        time::Duration,
    };

    const BASE_TEST_PORT: u16 = 43000;

    #[test]
    fn test_handler_write() {
        let test_port = BASE_TEST_PORT + 1;
        let expected = [48, 48, 48, 48, 48];

        // Setting up nc listener
        let mut ncat = if cfg!(target_os = "windows") {
            panic!() // Windows user can implement
        } else {
            Command::new("nc")
                .args(["-l", "-s", "127.0.0.1", "-p", &test_port.to_string()])
                .stdout(Stdio::piped())
                .spawn()
                .expect("Could not start")
        };
        thread::sleep(Duration::from_millis(250));
        let mut client_interface =
            TcpInterface::new_client("127.0.0.1".to_string(), test_port).unwrap();
        match TcpInterface::write(&mut client_interface, &expected) {
            Ok(n) => println!("Sent {} bytes", n),
            Err(why) => panic!("Failed to send bytes: {}", why),
        }
        thread::sleep(Duration::from_millis(250));

        // Checking if transfer was successful
        let nc_result = ncat.stdout.as_mut().unwrap();
        let mut read_buf: [u8; 5] = [0; 5];
        let status = nc_result.read_exact(&mut read_buf);
        match status {
            Err(why) => panic!("Failed to read from netcat: {}", why),
            Ok(_) => assert_eq!(read_buf, expected),
        }

        // Cleaning up resources
        ncat.kill().unwrap();
    }
    #[test]
    fn test_handler_read() {
        let test_port = BASE_TEST_PORT + 2;
        let test_input = "0xDEADBEEF";
        // Setting up nc server
        let mut ncat = if cfg!(target_os = "windows") {
            panic!() // Windows user can implement
        } else {
            Command::new("nc")
                .args(["-l", "-s", "127.0.0.1", "-p", &test_port.to_string()])
                .stdin(Stdio::piped())
                .spawn()
                .expect("Could not start")
        };

        thread::sleep(Duration::from_millis(250));

        let mut client_interface =
            TcpInterface::new_client("127.0.0.1".to_string(), test_port).unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];

        let send_nc = ncat.stdin.as_mut().unwrap();
        send_nc
            .write_all(test_input.as_bytes())
            .expect("Failed to write to nc");
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
        assert_eq!(
            str::from_utf8(&buffer[..test_input.len()]).expect("Failed to convert msg to &str"),
            test_input
        );
        ncat.kill().expect("Unable to kill netcat");
    }
}
