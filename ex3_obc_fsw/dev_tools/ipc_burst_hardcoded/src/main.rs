/*
Written by Devin Headrick 
Summer 2024

Setup Connection to IPC unix domain socket server (this acts as the client), and the 
send some hardcoded data. The issue is that before we were reading user input from std in, and sending that (ASCII)
data to the receiver, but now we need to send binary data. 

Usage:
    cargo run --bin ipc_burst_hardcoded <name of target> 

*/


use ipc_interface::{IPCInterface, IPC_BUFFER_SIZE, read_socket, send_over_socket};
use message_structure::{Msg, serialize_msg};
use common::*;

fn main(){
    // Get name of target from args 
    let args: Vec<String> = std::env::args().collect();
    
    //Setup interface for comm with OBC FSW components (IPC), by acting as a client connecting to msg dispatcher server
    let ipc_interface = IPCInterface::new_client("test_handler".to_string()).unwrap();
    let mut ipc_buf = vec![0; IPC_BUFFER_SIZE]; //Buffer to read incoming messages from IPC

    // Define msg to send contents
    let msg_data = vec![0x01, 0x03, 0x0a, 0x00];
    let msg_to_send = Msg::new(0,0x01, component_ids::COMS, 0x02, opcodes::COMS::GetHK as u8, msg_data);
    let msg_bytes = serialize_msg(&msg_to_send).unwrap(); 

    println!("Attempting to send: {:?}", msg_bytes);

    // Send the msg
    send_over_socket(ipc_interface.fd, msg_bytes).unwrap();

    println!("Sent successful");

    //Read the data back
    // let mut socket_buf = vec![0u8; IPC_BUFFER_SIZE];
    // let output = read_socket(ipc_interface.fd, &mut socket_buf).unwrap();
    // println!("Received: {:?}", socket_buf);


}
