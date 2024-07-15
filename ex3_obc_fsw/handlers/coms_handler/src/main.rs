/*
Written By Devin Headrick
Summer 2024

For tall thin all we want this to do is talk to this (via TCP) and have it relay its data to the message dispatcher (via IPC unix domain socket)

TODO - Detect if connection to either msg dispatcher or UHF transceiver is lost, and handle that - attempt to reconnect
TODO - implement a 'gs' connection flag, which the handler uses to determine whether or not it can downlink messages to the ground station.
TODO - mucho error handling
*/
use std::thread;
use std::time::Duration;
use common::component_ids::{COMS, GS};
use common::constants::UHF_MAX_MESSAGE_SIZE_BYTES;
use common::opcodes;
use common::ports;
use ipc_interface::{read_socket, send_over_socket, IPCInterface, IPC_BUFFER_SIZE};
use message_structure::{deserialize_msg, serialize_msg, Msg};
use std::vec;
use tcp_interface::{Interface, TcpInterface};
use bulk_msg_handler::*;

/// Setup function for decrypting incoming messages from the UHF transceiver
/// This just decrypts the bytes and does not return a message from the bytes
fn decrypt_bytes_from_gs(encrypted_bytes: &Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
    // TODO - Decrypt the message
    let decrypted_byte_vec = encrypted_bytes.clone();
    Ok(decrypted_byte_vec)
}

/// Write the provided arg data to the UHF beacon
fn set_beacon_value(new_beacon_value: Vec<u8>) {
    // TODO - write this data to the UHF beacon buffer (or however it actually works w/ the hardware)
    println!("Setting beacon value to: {:?}", new_beacon_value);
}

/// For messages directed FOR the coms handler directly. Based on the opcode of the message, perform some action
fn handle_msg_for_coms(msg: &Msg) {
    let opcode = msg.header.op_code;
    match opcode {
        opcodes::coms::GET_HK => {
            println!("Opcode 3: Get House Keeping Data from COMS Handler for UHF");
        }
        opcodes::coms::SET_BEACON => {
            println!("Opcode 4: Set the Beacon value");
            //TODO - for now just get the msg body (data) and write that to the beacon
            set_beacon_value(msg.msg_body.clone());
        }
        _ => println!("Invalid msg opcode"),
    }
}

/// Fxn to write the a msg to the UHF transceiver for downlinking
fn handle_msg_for_gs(msg: &Msg, interface: &mut TcpInterface) -> Result<(), std::io::Error> {
    let msg_len = msg.msg_body.len() + 5; // account for header bytes
    if msg_len > UHF_MAX_MESSAGE_SIZE_BYTES.into() {
        // If the message is a bulk message, then fragment it before downlinking
        let messages: Vec<Msg> = handle_large_msg(msg.clone())?;
        let serialized_first_msg: Vec<u8> = serialize_msg(&messages[0])?;
        interface.send(&serialized_first_msg);

        // TODO - wait for gs to respond with ACK to send next messages
        println!("About to send {} messages", messages.len());
        thread::sleep(Duration::from_secs(10));
        for i in 1..messages.len() {
            let serialized_msg_packet: Vec<u8> = serialize_msg(&messages[i])?;
            interface.send(&serialized_msg_packet)?;
        }
    } else {
    // TMP - Writes directly to gs. will write to sim UHF when done
    let serialized_msg: Vec<u8> = serialize_msg(&msg)?;
    interface.send(&serialized_msg)?;
    }
    Ok(())
}

/// Handle incomming messages from other OBC FSW components
/// Determines based on msg destination where to send it
fn handle_ipc_msg(msg: Msg, interface: &mut TcpInterface) {
    // Check if the message is destined for the coms handler directly, or to be downlinked to the ground station
    let destination = msg.header.dest_id;
    match destination {
        COMS => handle_msg_for_coms(&msg),
        GS => handle_msg_for_gs(&msg, interface).unwrap(),
        _ => {
            println!("Invalid msg destination from IPC read");
        }
    }
}

/// Handle incomming messages from the UHF transceiver (uplinked stuff)
/// Determines based on msg destination where to send it
fn handle_uhf_msg(msg: &Msg, ipc_interface_fd: i32) {
    // Check if the message is destined for the coms handler directly, or to be downlinked to the ground station
    let destination = msg.header.dest_id;
    match destination {
        COMS => handle_msg_for_coms(msg),
        _ => {
            // TODO - Send message to msg dispatcher via IPC connection
            println!("Sending msg to msg_dispatcher");
            let serialized_msg_result = serialize_msg(msg).unwrap();
            send_over_socket(ipc_interface_fd, serialized_msg_result).unwrap();
        }
    }
}

/// All things to be downlinked use this fxn (later on we want a sort of buffer to store what was downlinked until we get confirmation from the GS it was recevied)
/// This will handle logging all messages attempted to be downlinked, and handle errors associated with writing data to the UHF transceiver for downlink
fn write_msg_to_uhf_for_downlink(interface: &mut TcpInterface, msg: Msg) {
    let serialized_msg_result = serialize_msg(&msg);
    match serialized_msg_result {
        Ok(serialized_msg) => {
            let send_result = interface.send(&serialized_msg);
            match send_result {
                Ok(_) => {
                    // Successfully sent the message
                    println!("Successfully sent msg to uhf transceiver: {:?}", msg);
                }
                Err(e) => {
                    // Handle the error when sending the message
                    println!("Error sending msg to uhf: {:?}", e);
                }
            }
        }
        Err(e) => {
            // Handle the error when serializing the message
            println!("Error serializing message: {:?}", e);
        }
    }
}

fn main() {
    println!("Beginning Coms Handler...");

    //Setup interface for comm with UHF transceiver [ground station] (TCP for now)
    let mut tcp_interface =
        TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_COMMS_PORT).unwrap();

    //Setup interface for comm with OBC FSW components (IPC), by acting as a client connecting to msg dispatcher server
    let ipc_interface_res = IPCInterface::new("coms_handler".to_string());
    if ipc_interface_res.is_err() {
        println!("Error creating IPC interface: {:?}", ipc_interface_res.err());
        return;
    }
    let ipc_interface = ipc_interface_res.unwrap();

    let mut ipc_buf = vec![0; IPC_BUFFER_SIZE]; //Buffer to read incoming messages from IPC
    let mut ipc_num_bytes_read = 0;

    let mut uhf_buf = vec![0; UHF_MAX_MESSAGE_SIZE_BYTES as usize]; //Buffer to read incoming messages from UHF
    let mut uhf_num_bytes_read = 0;

    loop {
        // Poll both the UHF transceiver and IPC unix domain socket
        let ipc_bytes_read_result = read_socket(ipc_interface.fd, &mut ipc_buf);
        match ipc_bytes_read_result {
            Ok(num_bytes_read) => {
                ipc_num_bytes_read = num_bytes_read;
                println!("Read {} bytes", ipc_num_bytes_read);
            }
            Err(e) => {
                println!("Error reading from IPC socket: {:?}", e);
            }
        }

        if ipc_num_bytes_read > 0 {
            println!("Received IPC Msg bytes");
            let deserialized_msg_result = deserialize_msg(&ipc_buf.as_slice());
            match deserialized_msg_result {
                Ok(deserialized_msg) => {
                    println!("Dserd msg body len {}", deserialized_msg.msg_body.len());
                    handle_ipc_msg(deserialized_msg, &mut tcp_interface);
                }
                Err(e) => {
                    println!("Error deserializing IPC msg: {:?}", e);
                    //Handle deserialization of IPC msg failure
                }
            };
        }

        let uhf_bytes_read_result = tcp_interface.read(&mut uhf_buf);
        match uhf_bytes_read_result {
            Ok(num_bytes_read) => {
                uhf_num_bytes_read = num_bytes_read;
            }
            Err(e) => {
                println!("Error reading from UHF transceiver: {:?}", e);
            }
        }

        if uhf_num_bytes_read > 0 {
            println!("Received UHF Msg bytes");
            let mut ack_msg_id = 0;
            let mut ack_msg_body = vec![0x4F, 0x4B]; // 0x4F = O , 0x4B = K  [OK
                                                     //TODO - Decrypt incomming encrypted bytes
            let decrypted_byte_result = decrypt_bytes_from_gs(&uhf_buf);
            match decrypted_byte_result {
                Ok(decrypted_byte_vec) => {
                    let deserialized_msg_result = deserialize_msg(&decrypted_byte_vec.as_slice());
                    match deserialized_msg_result {
                        Ok(deserialized_msg) => {
                            handle_uhf_msg(&deserialized_msg, ipc_interface.fd);
                            ack_msg_id = deserialized_msg.header.msg_id;
                        }
                        Err(e) => {
                            println!("Error deserializing msg from UHF: {:?}", e);
                            ack_msg_body = vec![
                                0x45, 0x52, 0x52, 0x2D, 0x6D, 0x73, 0x67, 0x20, 0x64, 0x65, 0x73,
                                0x65, 0x72, 0x69, 0x61, 0x6C, 0x69, 0x7A, 0x61, 0x74, 0x69, 0x6F,
                                0x6E, 0x20, 0x66, 0x61, 0x69, 0x6C, 0x65, 0x64,
                            ]; // [ERR-msg deserialization failed]
                        }
                    };
                }
                Err(e) => {
                    println!("Error decrypting bytes from UHF: {:?}", e);
                    ack_msg_body = vec![
                        0x45, 0x52, 0x52, 0x2D, 0x6D, 0x73, 0x67, 0x20, 0x64, 0x65, 0x63, 0x72,
                        0x79, 0x70, 0x74, 0x69, 0x6F, 0x6E, 0x20, 0x66, 0x61, 0x69, 0x6C, 0x65,
                        0x64,
                    ]; // [ERR-msg decryption failed]
                }
            };

            //EMIT AN ACK TO TELL SENDER WE RECEIVED THE MSG
            // OK -> if decryption and msg deserialization of bytes succeeds
            // ERR -> If decryption fails or msg deserialization fails (inform sender what failed)
            let ack_msg = Msg::new(0,ack_msg_id, GS, COMS, 200, ack_msg_body);
            write_msg_to_uhf_for_downlink(&mut tcp_interface, ack_msg);
            // uhf_buf.clear(); //FOR SOME REASON CLEARING THE BUFFERS WOULD CAUSE THE CONNECTION TO DROP AFTER A SINGLE MSG IS READ
        }
    }
}

#[cfg(test)]
mod tests {}
