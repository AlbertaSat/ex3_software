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
use ipc::*;
use message_structure::{deserialize_msg, serialize_msg, Msg, MsgType};
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
fn handle_bulk_msg_for_gs(msg: &Msg, interface: &mut TcpInterface) -> Result<(), std::io::Error> {
    
    // Slice Msg before downlinking
    let messages: Vec<Msg> = handle_large_msg(msg.clone())?;
    // Send first Msg
    write_msg_to_uhf_for_downlink(interface, messages[0].clone());

    // TODO - wait for gs to respond with ACK to send next messages

    println!("About to send {} messages", messages.len());
    thread::sleep(Duration::from_secs(10));
    for i in 1..messages.len() {
        write_msg_to_uhf_for_downlink(interface, messages[i].clone());
    }
    
    Ok(())
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

    //Setup interface for comm with OBC FSW components (IPC), for the purpose of passing messages to and from the GS
    let ipc_gs_interface_res = IpcClient::new("gs_bulk".to_string());
    if ipc_gs_interface_res.is_err() {
        println!("Error creating IPC interface: {:?}", ipc_gs_interface_res.err());
        return;
    }
    let mut ipc_gs_interface = ipc_gs_interface_res.unwrap();

    //Setup interface for comm with OBC FSW components (IPC), for passing messages to and from the UHF specifically
    // TODO - name this to gs_handler once uhf handler and gs handler are broken up from this program.
    // Will have to be changed in msg_dispatcher as well
    let ipc_coms_interface_res = IpcClient::new("coms_handler".to_string());
    if ipc_coms_interface_res.is_err() {
        println!("Error creating IPC interface: {:?}", ipc_coms_interface_res.err());
        return;
    }

    let mut ipc_coms_interface = ipc_coms_interface_res.unwrap();

    let mut uhf_buf = vec![0; UHF_MAX_MESSAGE_SIZE_BYTES as usize]; //Buffer to read incoming messages from UHF
    let mut uhf_num_bytes_read = 0;

    loop {
        // Poll both the UHF transceiver and IPC unix domain socket for the GS channel
        let mut clients = vec![&mut ipc_gs_interface, &mut ipc_coms_interface];
        let _ = poll_ipc_clients(&mut clients);

        if ipc_gs_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
            println!("Received IPC Msg bytes for GS");
            let deserialized_msg_result = deserialize_msg(&ipc_gs_interface.buffer);
            match deserialized_msg_result {
                Ok(deserialized_msg) => {
                    println!("Dserd msg body len {}", deserialized_msg.msg_body.len());
                    // writes directly to GS, handling case if it's a bulk message
                    if deserialized_msg.header.msg_type == MsgType::Bulk as u8 {
                        let _ = handle_bulk_msg_for_gs(&deserialized_msg, &mut tcp_interface);
                    } else {
                        let _ = write_msg_to_uhf_for_downlink(&mut tcp_interface, deserialized_msg);
                    }
                }
                Err(e) => {
                    println!("Error deserializing GS IPC msg: {:?}", e);
                    //Handle deserialization of IPC msg failure
                }
            };
        }


        // Poll the IPC unix domain socket for the COMS channel
        if ipc_coms_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
            println!("Received COMS IPC Msg bytes");
            let deserialized_msg_result = deserialize_msg(&ipc_coms_interface.buffer);
            match deserialized_msg_result {
                Ok(deserialized_msg) => {
                    println!("Dserd msg body len {}", deserialized_msg.msg_body.len());
                    // Handles msg internally for COMS
                    handle_msg_for_coms(&deserialized_msg);
                }
                Err(e) => {
                    println!("Error deserializing COMS IPC msg: {:?}", e);
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
            println!("Received bytes from UHF");
            let mut ack_msg_id = 0;
            let mut ack_msg_body = vec![0x4F, 0x4B]; // 0x4F = O , 0x4B = K  [OK
                                                     //TODO - Decrypt incomming encrypted bytes
            let decrypted_byte_result = decrypt_bytes_from_gs(&uhf_buf);
            match decrypted_byte_result {
                // After decrypting, send directly to the msg_dispatcher
                Ok(decrypted_byte_vec) => {
                    let _ = ipc_write(ipc_coms_interface.fd, &decrypted_byte_vec);
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
