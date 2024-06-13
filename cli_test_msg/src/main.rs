/**
 * Written by Rowan Rasmusson
 * 2024 summer
 *
 * Simple CLI to write commands to the OBC via TCP. This establishes a TCP client and sends data as bytes
 */

 use std::env;
 use std::io::Write;
 use std::net::Ipv4Addr;
 use std::net::TcpStream;
 use message_structure::*;
 use serde_json;

fn main() {
    println!("Writing data to OBC FSW via TCP client socket connection");
    let args: Vec<String> = env::args().collect();

    if args.len() < 1 {
        println!("Usage: <obc_port> Default_Msg:<data> ...");
        return;
    }

    let port = args[1].parse::<u16>().unwrap();

    let data: Msg = Msg::new(0,0,0,0,vec![1,1,1,1,1,1]);

    let mut stream = TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), port)).unwrap();
    let output_stream = &mut stream;

    let command_bytes = build_command_bytes(data);

    output_stream.write(&command_bytes).unwrap();
    output_stream.flush().unwrap();
}

fn build_command_bytes(data: Msg) -> Vec<u8> {
    let mut buf = Vec::new();
    let _serialized_msg = serde_json::to_writer(&mut buf, &data).unwrap();

    println!("Command Byte Values: {:?}", buf);
    buf
}
