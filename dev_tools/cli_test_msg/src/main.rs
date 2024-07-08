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
use cli_test_msg::timestamp_to_epoch;
use message_structure::*;

fn main() {
    println!("Writing data to OBC FSW via TCP client socket connection");
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: <obc_port> <subsystem> ");
        return;
    }

    let port = args[1].parse::<u16>().unwrap();

    let subsystem: &String = &args[2];
    // Default DFGM msg
    let mut data: Msg = Msg::new(0,0,0,0,vec![]);
    if subsystem == "scheduler" {
        let timestamp: &String = &args[3];

        // time in format YYYY-MM-DD HH:MM:SS
        let msg_time: u64 = timestamp_to_epoch(timestamp.clone()).unwrap();
        let msg_time_bytes = msg_time.to_le_bytes().to_vec();

        let mut data: Msg = Msg::new(22,3,25,25,msg_time_bytes.clone());
        let inner_msg: Msg = Msg::new(22,4,0,0,msg_time_bytes);
        let serialized_inner_msg = serialize_msg(&inner_msg).unwrap();
        data = Msg::new(0,8,1,2,serialized_inner_msg);
    } else if subsystem == "dfgm" {
        data = Msg::new(0,3,0,0,vec![0]);
    }

    let mut stream = TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), port)).unwrap();
    let output_stream = &mut stream;

    let command_bytes = serialize_msg(&data).unwrap();
    println!("Bytes Sent: {:?}", command_bytes);

    output_stream.write(&command_bytes).unwrap();
    output_stream.flush().unwrap();
}
