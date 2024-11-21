/*
Written by Devin Headrick 
Summer 2024

Setup Connection to IPC unix domain socket server (this acts as the client), and the 
send some hardcoded data. The issue is that before we were reading user input from std in, and sending that (ASCII)
data to the receiver, but now we need to send binary data. 

Usage:
    cargo run --bin ipc_burst_hardcoded <name of target> 

*/


use ipc::*;
use common::message_structure::{Msg, serialize_msg};
use common::*;

fn main() {
    //Setup interface for comm with OBC FSW components (IPC), by acting as a client connecting to msg dispatcher server
    let ipc_interface = IpcClient::new("test_handler".to_string()).unwrap();

    // Define msg to send contents
    let msg_data = vec![0x01, 0x03, 0x0a, 0x00];
    let msg_to_send = Msg::new(0,0x01, ComponentIds::COMS as u8, 0x02, opcodes::COMS::GetHK as u8, msg_data);
    let msg_bytes = serialize_msg(&msg_to_send).unwrap(); 

    println!("Attempting to send: {:?}", msg_bytes);

    // Send the msg
    ipc_write(&ipc_interface.fd, &msg_bytes).unwrap();

    println!("Sent successful");

    //Read the data back
    // let mut socket_buf = vec![0u8; IPC_BUFFER_SIZE];
    // let output = read_socket(ipc_interface.fd, &mut socket_buf).unwrap();
    // println!("Received: {:?}", socket_buf);


}
