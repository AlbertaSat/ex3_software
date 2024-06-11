use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn handle_client(mut stream: TcpStream) {
    let message = b"Mock dispatcher message\n";
    
    // Simulate some delay
    // thread::sleep(Duration::from_secs(1));
    // stream.write_all(message).unwrap();
    stream.flush().unwrap(); // Ensure the message is sent immediately

    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
                stream.write_all(&buffer[..n]).unwrap();
                stream.flush().unwrap(); // Ensure the message is sent immediately
            }
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1900")?;
    println!("Server listening on port 1900");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}
