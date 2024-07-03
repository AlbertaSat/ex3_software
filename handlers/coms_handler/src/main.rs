/*
Written By Devin Headrick
Summer 2024

TODO - implement a 'gs' connection flag, which the handler uses to determine whether or not it can downlink messages to the ground station.

TODO - mucho error handling
*/

use common::component_ids::{COMS, GS};
use common::constants::UHF_MAX_MESSAGE_SIZE_BYTES;
use message_structure::Msg;

///Setup function for decrypting incoming messages from the UHF transceiver
pub fn decrypt_byte_from_gs(encrypted_bytes: Vec<u8>) -> Vec<u8> {
    // TODO - Decrypt the message
    let mut decrypted_bytes = Vec::new();

    return decrypted_bytes;
}
/// For messages directed FOR the coms handler directly. Based on the opcode of the message, perform some action
/// Later this is where we will want to allow the OBC to do things like update the Beacon contents etc
fn handle_msg_for_coms(msg: Msg) {
    let opcode = msg.header.op_code;
    match (opcode) {
        0 => println!("Opcode 0: Do nothing"),
        1 => println!("Opcode 1: Do something"),
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
fn handle_uhf_msg(msg: Msg) {
    // Check if the message is destined for the coms handler directly, or to be downlinked to the ground station
    let destination = msg.header.dest_id;
    match (destination) {
        COMS => handle_msg_for_coms(msg),
        _ => {
            //TODO - check if bulk message - if so then handle it

            // TODO - Send message to msg dispatcher via IPC connection
        }
    }
}

/*
When reading from UHF -> If we have received something:
- Emit an 'ack' that tells sender we got something
- Decrypt bytes
- Deserialize the bytes (create message obj from bytes)
- Check the message destination
    - if it is not for the coms handler directly, then forward it to the message dispatcher (write to IPC connection to message dispatcher)
    - If it is for the coms handler directly, then handle it based on op code

*/

/*
When reading from IPC -> If we received something:
- Deserialize the bytes (create message obj from bytes)
- Check message destination
    - Not for coms hanlder direclty, then
        - Check if the message needs to fragmented
            - If not bulk msg: write it to UHF transceiver (downlink it)
            - If bulk msg: 'handle it as a bulk msg' -> then
    - For coms handler directly, then handle it based on op code

*/

fn main() {
    println!("Beginning Coms Handler...");

    //Setup interface for comm with UHF transceiver [ground station] (TCP for now)

    //Setup interface for comm with OBC FSW components (IPC)

    //loop - polling listen to both UHF transceiver & IPC unix domain socket
    loop {
        //Poll both the UHF transceiver and IPC unix domain socket
    }
}
