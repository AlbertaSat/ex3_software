/*
Written By Devin Headrick
Summer 2024

For tall thin all we want this to do is talk to this (via TCP) and have it relay its data to the message dispatcher

When reading from UHF -> If we have received something:
- Emit an 'ack' that tells sender we got something
- Decrypt bytes
- Deserialize the bytes (create message obj from bytes)
- Check the message destination
    - if it is not for the coms handler directly, then forward it to the message dispatcher (write to IPC connection to message dispatcher)
    - If it is for the coms handler directly, then handle it based on op code


When reading from IPC -> If we received something:
- Deserialize the bytes (create message obj from bytes)
- Check message destination
    - Not for coms hanlder direclty, then
        - Check if the message needs to fragmented
            - If not bulk msg: write it to UHF transceiver (downlink it)
            - If bulk msg: 'handle it as a bulk msg' -> then
    - For coms handler directly, then handle it based on op code

TODO - implement a 'gs' connection flag, which the handler uses to determine whether or not it can downlink messages to the ground station.

TODO - mucho error handling

*/

use common::component_ids::{COMS, GS};
use common::constants::UHF_MAX_MESSAGE_SIZE_BYTES;
use common::opcodes;
use ipc_interface::{read_socket, send_over_socket, IPCInterface, IPC_BUFFER_SIZE};
use message_structure::{deserialize_msg, serialize_msg, Msg};

/// Setup function for decrypting incoming messages from the UHF transceiver
/// This just decrypts the bytes and does not return a message from the bytes
fn decrypt_bytes_from_gs(encrypted_bytes: Vec<u8>) -> Vec<u8> {
    // TODO - Decrypt the message
    let mut decrypted_bytes = Vec::new();

    return decrypted_bytes;
}

/// Write the provided arg data to the UHF beacon
fn set_beacon_value(new_beacon_value: Vec<u8>) {
    // TODO - write this data to the UHF beacon buffer
    println!("Setting beacon value to: {:?}", new_beacon_value);
}

/// For messages directed FOR the coms handler directly. Based on the opcode of the message, perform some action
/// Later this is where we will want to allow the OBC to do things like update the Beacon contents etc
fn handle_msg_for_coms(msg: Msg) {
    let opcode = msg.header.op_code;
    match (opcode) {
        opcodes::coms::GET_HK => {
            println!("Opcode 3: Get House Keeping Data from COMS Handler for UHF");

        }
        opcodes::coms::SET_BEACON => {
            println!("Opcode 4: Set the Beacon value");
            //TODO - for now just get the msg body (data) and write that to the beacon
            set_beacon_value(msg.msg_body);
        }
        _ => println!("Invalid msg opcode"),
    }
}

/// Fxn to write the a msg to the UHF transceiver for downlinking
fn handle_msg_for_gs(msg: Msg) {
    let msg_len = msg.header.msg_len;
    if msg_len > UHF_MAX_MESSAGE_SIZE_BYTES {
        // If the message is a bulk message, then fragment it before downlinking
        // TODO - handle bulk message
    }
    // TODO - downlink message to ground station
}

/// Handle incomming messages from other OBC FSW components
/// Determines based on msg destination where to send it
fn handle_ipc_msg(msg: Msg) {
    // Check if the message is destined for the coms handler directly, or to be downlinked to the ground station
    let destination = msg.header.dest_id;
    match (destination) {
        COMS => handle_msg_for_coms(msg),
        GS => handle_msg_for_gs(msg),
        _ => {
            println!("Invalid msg destination from IPC read");
        }
    }
}

/// Handle incomming messages from the UHF transceiver (uplinked stuff)
/// Determines based on msg destination where to send it
fn handle_uhf_msg(msg: Msg) {
    // Check if the message is destined for the coms handler directly, or to be downlinked to the ground station
    let destination = msg.header.dest_id;
    match (destination) {
        COMS => handle_msg_for_coms(msg),
        _ => {
            // TODO - Send message to msg dispatcher via IPC connection
        }
    }
}

// All things to be downlinked use this fxn (later on we want a sort of buffer to store what was downlinked until we get confirmation from the GS it was recevied)
fn downlink_msg_to_gs (msg: Msg) {
    //TODO - write the msg to the UHF transceiver

}


fn main() {
    println!("Beginning Coms Handler...");

    //Setup interface for comm with UHF transceiver [ground station] (TCP for now)
    //TODO -

    //Setup interface for comm with OBC FSW components (IPC), by acting as a client connecting to msg dispatcher server
    let ipc_interface = IPCInterface::new("coms_handler".to_string());

    let mut ipc_buf = vec![0; IPC_BUFFER_SIZE]; //Buffer to read incoming messages from IPC

    let mut uhf_buf = vec![0; UHF_MAX_MESSAGE_SIZE_BYTES as usize]; //Buffer to read incoming messages from UHF

    loop {
        //Poll both the UHF transceiver and IPC unix domain socket
        let ipc_bytes_read = read_socket(ipc_interface.fd, &mut ipc_buf).unwrap();

        if ipc_bytes_read > 0 {
            println!("Received IPC Msg bytes: {:?}", ipc_buf);
            let msg = deserialize_msg(&ipc_buf).unwrap();
            handle_ipc_msg(msg);
            //TODO clear the buffer 
        }

        // let uhf_bytes = 0;
        
        // if uhf_bytes > 0 {
        //     println!("Received UHF Msg bytes: {:?}", uhf_bytes);
        //     //TODO - EMIT AN ACK to inform the sender (gs) that we got the message
        //     let decrypted_bytes = decrypt_bytes_from_gs(uhf_bytes);
        //     let msg = deserialize_msg(&decrypted_bytes).unwrap();   
        //     handle_uhf_msg(msg);
        //     //TODO clear the buffer 
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
