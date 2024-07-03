/*
Written by Devin Headrick
Summer 2024
*/

use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

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
        Err(Error::new(
            std::io::ErrorKind::Other,
            "No incoming connections",
        ))
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