/*
Written by Devin Headrick
Summer 2024
*/

use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::thread;

const TCP_BUFFER_SIZE: usize = 1024;

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

/// handle asynchronous reading on an interface by spawing a new thread and passing data to mpsc sender channel
#[allow(dead_code)]
pub fn async_read<T: Interface + Send + 'static>(mut interface: T, sender: Sender<Vec<u8>>, buffer_size: usize) {
    thread::spawn(move || {
        println!("Async read thread started");
        loop {
            let mut buffer = vec![0; buffer_size];
            match interface.read(&mut buffer) {
                Ok(_) => {
                    //if buffer is empty or only zeroes, ignore it
                    if buffer.iter().all(|&x| x == 0) {
                        break;
                    }
                    sender.send(buffer).unwrap();
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
    });
}

/// handle asynchronous writing on an interface by spawing a new thread and reading data from mpsc receiver channel
#[allow(dead_code)]
pub fn async_write<T: Interface + Send + 'static>(mut interface: T, receiver: Receiver<Vec<u8>>) {
    thread::spawn(move || {
        println!("Async write thread started");
        for data in receiver {
            if let Err(e) = interface.send(&data) {
                eprintln!("Write error: {}", e);
                break;
            } else {
                //println!("Data sent: {:?}", data);
            }
        }
    });
}

pub fn presence() -> String {
    "interfaces".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_tcp_interface() {
        let ip = "127.0.0.1".to_string();
        let port = 1802;
        let tcp_interface = TcpInterface::new_client(ip, port).unwrap();

        let (send_tx, send_rx) = mpsc::channel();
        let (recv_tx, recv_rx) = mpsc::channel();

        async_read(tcp_interface.clone(), recv_tx, 1024);
        async_write(tcp_interface.clone(), send_rx);

        send_tx.send(b"Hello, World!".to_vec());

        if let Ok(data) = recv_rx.recv() {
            println!("Received data: {:?}", data);
        }

        // Sleep to let threads run
        thread::sleep(Duration::from_secs(3));
    }

    #[test]
    fn test_tcp_interface_server(){
        let ip = "127.0.0.1";
        let port = 1900;
        let tcp_interface_server = TcpInterface::new_server(ip.to_string(), port).unwrap();

        // Create channels for talking to reading / writing threads
        let (send_tx, send_rx) = mpsc::channel();
        let (recv_tx, recv_rx) = mpsc::channel();

        async_read(tcp_interface_server.clone(), recv_tx, 1024);
        async_write(tcp_interface_server.clone(), send_rx);

        // Wait for a connection to be established - check if connection is established instead of waiting 

        send_tx.send(b"Hello, World!".to_vec());

        if let Ok(data) = recv_rx.recv() {
            println!("Received data: {:?}", data);
        }

        // Sleep to let threads run
        thread::sleep(Duration::from_secs(3));

    }
}
