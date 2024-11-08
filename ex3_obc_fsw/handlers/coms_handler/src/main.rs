/*
Written By Devin Headrick
Summer 2024

For tall thin all we want this to do is talk to this (via TCP) and have it relay its data to the message dispatcher (via IPC unix domain socket)

TODO - Detect if connection to either msg dispatcher or UHF transceiver is lost, and handle that - attempt to reconnect
TODO - implement a 'gs' connection flag, which the handler uses to determine whether or not it can downlink messages to the ground station.
TODO - mucho error handling
*/
use log::{debug, trace, warn};
use logging::*;

use common::component_ids::{ComponentIds, COMS, GS};
use common::constants::UHF_MAX_MESSAGE_SIZE_BYTES;
use common::opcodes;
use common::ports;
use ipc::*;
use message_structure::{deserialize_msg, serialize_msg, Msg, MsgType};
use std::os::fd::OwnedFd;
use std::vec;
use tcp_interface::{Interface, TcpInterface};
mod uhf_handler;
use uhf_handler::UHFHandler;

/// Setup function for decrypting incoming messages from the UHF transceiver
/// This just decrypts the bytes and does not return a message from the bytes
fn decrypt_bytes_from_gs(encrypted_bytes: &[u8]) -> Result<&[u8], std::io::Error> {
    // TODO - Decrypt the message
    let decrypted_byte_vec = encrypted_bytes;
    Ok(decrypted_byte_vec)
}

/// Write the provided arg data to the UHF beacon
fn set_beacon_value(new_beacon_value: Vec<u8>) {
    // TODO - write this data to the UHF beacon buffer (or however it actually works w/ the hardware)
    trace!("Setting beacon value to: {:?}", new_beacon_value);
}

/// For messages directed FOR the coms handler directly. Based on the opcode of the message, perform some action
fn handle_msg_for_coms(msg: &Msg) {
    let opcode_enum = opcodes::COMS::from(msg.header.op_code);
    match opcode_enum {
        opcodes::COMS::GetHK => {
            trace!("Opcode 3: Get House Keeping Data from COMS Handler for UHF");
        }
        opcodes::COMS::SetBeacon => {
            trace!("Opcode 4: Set the Beacon value");
            //TODO - for now just get the msg body (data) and write that to the beacon
            set_beacon_value(msg.msg_body.clone());
        }
        _ => debug!("Invalid msg opcode"),
    }
}

/// Function to send the initial messages containing num of 4KB msgs to expect and the number of
/// data bytes to expect once the msg is rebuilt
fn send_initial_bulk_to_gs(initial_msg: Msg, interface: &mut TcpInterface) {
    write_msg_to_uhf_for_downlink(interface, initial_msg);
}

/// Fxn to write the a msg to the UHF transceiver for downlinking. It will wait to receive an ACK
/// before sending the msgs down to the GS.
/// It expects a mesg to send to the GS. It also needs a messages that is send from the Bulk Msg Dispatcher
/// that contains the number of 4KB msgs and number of data bytes total.
/// Also sends the messages to the UHF/GS
// fn handle_bulk_msg_for_gs(msg: Msg, interface: &mut TcpInterface) -> Result<(), std::io::Error> {
//     thread::sleep(Duration::from_secs(2));
//     for i in 0..messages.len() {
//         let cur_msg = messages[i].clone();
//         // Handle_large_msg puts another 'header' msg at the beginning of the Vec<Msg> saying how many bulk msgs there are.
//         let msgs_to_send = handle_large_msg(cur_msg, DONWLINK_MSG_BODY_SIZE)?;

//         for j in 0..msgs_to_send.len() {
//             write_msg_to_uhf_for_downlink(interface, msgs_to_send[j].clone());
//             thread::sleep(Duration::from_millis(10));
//         }
//     }

//     Ok(())
// }

/// Function for sending an ACK to the bulk disp letting it know to send bulk msgs for downlink
fn send_bulk_ack(fd: &OwnedFd) -> Result<(), std::io::Error> {
    let ack_msg = Msg::new(
        MsgType::Ack as u8,
        20,
        ComponentIds::BulkMsgDispatcher as u8,
        ComponentIds::COMS as u8,
        0,
        vec![0],
    );
    ipc_write(fd, &serialize_msg(&ack_msg)?)?;
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
                    trace!("Successfully sent msg to uhf transceiver.");
                }
                Err(e) => {
                    // Handle the error when sending the message
                    debug!("Error sending msg to uhf: {:?}", e);
                }
            }
        }
        Err(e) => {
            // Handle the error when serializing the message
            debug!("Error serializing message: {:?}", e);
        }
    }
}

fn main() {
    let log_path = "ex3_obc_fsw/handlers/coms_handler/logs";
    init_logger(log_path);
    trace!("Logger initialized");
    trace!("Beginning Coms Handler...");

    // Setup interface for comm with OBC FSW components (IPC), for passing messages to and from the UHF specifically
    let ipc_coms_interface_res = IpcServer::new("COMS".to_string());
    let mut ipc_coms_interface = match ipc_coms_interface_res {
        Ok(i) => Some(i),
        Err(e) => {
            warn!("Cannot create COMS pipeline: {e}");
            None
        }
    };

    // Interface for IPC of cmd_dispatcher cmds that get sent up with a certain destination
    let ipc_cmd_interface_res = IpcServer::new("cmd_dispatcher".to_string());
    let mut ipc_cmd_interface = match ipc_cmd_interface_res {
        Ok(i) => Some(i),
        Err(e) => {
            warn!("Cannot create COMS pipeline: {e}");
            None
        }
    };

    //Setup interface for comm with OBC FSW components (IPC), for the purpose of passing messages to and from the GS
    let ipc_gs_interfac_res = IpcClient::new("gs_bulk".to_string());
    let mut ipc_gs_interface = match ipc_gs_interfac_res {
        Ok(i) => Some(i),
        Err(e) => {
            warn!("Cannot connect to bulk interface: {e}");
            None
        }
    };

    let mut gs_interface_non_bulk: Option<IpcServer> =
        match IpcServer::new("gs_non_bulk".to_string()) {
            Ok(server) => Some(server),
            Err(e) => {
                warn!("Error creating server to collect messages for ground station: {e}");
                None
            }
        };

    // Initialize ipc interface for UHF handler
    let ipc_uhf_interface_res = IpcServer::new("UHF".to_string());
    let mut ipc_uhf_interface = match ipc_uhf_interface_res {
        Ok(i) => Some(i),
        Err(e) => {
            warn!("Cannot create UHF handler pipeline: {e}");
            None
        }
    };

    // Initialize UHF handler struct
    let mut uhf_handler = UHFHandler::new();

    std::thread::sleep(std::time::Duration::from_secs(1));
    //Setup interface for comm with UHF transceiver [ground station] (TCP for now)
    let mut tcp_interface =
        match TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_COMMS_PORT) {
            Ok(tcp) => Some(tcp),
            Err(e) => {
                warn!("Error creating UHF interface: {e}");
                None
            }
        };

    let mut uhf_buf = vec![0; UHF_MAX_MESSAGE_SIZE_BYTES as usize]; //Buffer to read incoming messages from UHF
    let mut uhf_num_bytes_read = 0;
    let mut received_bulk_ack = false;
    let mut bulk_msgs_read = 0;
    let mut expected_msgs = 0;

    loop {
        uhf_buf.fill(0);
        // Poll both the UHF transceiver and IPC unix domain socket for the GS channel
        let mut clients = vec![&mut ipc_gs_interface];
        let mut servers = vec![
            &mut ipc_coms_interface,
            &mut ipc_cmd_interface,
            &mut ipc_uhf_interface,
        ];
        poll_ipc_server_sockets(&mut servers);
        let ipc_bytes_read_res = poll_ipc_clients(&mut clients);

        if let Some(ref mut init_ipc_gs_interface) = ipc_gs_interface {
            if init_ipc_gs_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
                trace!("Received IPC Msg bytes for GS");
                let deserialized_msg_result = deserialize_msg(&init_ipc_gs_interface.buffer);
                match deserialized_msg_result {
                    Ok(deserialized_msg) => {
                        // writes directly to GS, handling case if it's a bulk message
                        if deserialized_msg.header.msg_type == MsgType::Bulk as u8
                            && !received_bulk_ack
                        {
                            // If we haven't received Bulk ACK, we need to send ack
                            if let Some(e) = send_bulk_ack(&init_ipc_gs_interface.fd).err() {
                                println!("failed to send bulk ack: {e}");
                            }
                            received_bulk_ack = true;
                            let expected_msgs_bytes =
                                [deserialized_msg.msg_body[0], deserialized_msg.msg_body[1]];
                            expected_msgs = u16::from_le_bytes(expected_msgs_bytes);
                            trace!("Expecting {} 4KB msgs", expected_msgs);
                            // Send msg containing num of 4KB msgs and num of bytes to expect
                            send_initial_bulk_to_gs(
                                deserialized_msg,
                                tcp_interface.as_mut().unwrap(),
                            );
                        } else if deserialized_msg.header.msg_type == MsgType::Bulk as u8
                            && received_bulk_ack
                        {
                            // await_ack_for_bulk(&mut tcp_interface);
                            // Here where we read incoming bulk msgs from bulk_msg_disp
                            if bulk_msgs_read < expected_msgs {
                                if let Ok((ipc_bytes_read, ipc_name)) = ipc_bytes_read_res {
                                    if ipc_name.contains("gs") {
                                        let cur_buf =
                                            init_ipc_gs_interface.buffer[..ipc_bytes_read].to_vec();
                                        println!("Bytes read: {}", cur_buf.len());
                                        let cur_msg = deserialize_msg(&cur_buf).unwrap();
                                        write_msg_to_uhf_for_downlink(
                                            tcp_interface.as_mut().unwrap(),
                                            cur_msg,
                                        );
                                        bulk_msgs_read += 1;
                                    }
                                } else {
                                    warn!("Error reading bytes from poll.");
                                }
                            }
                        } else {
                            write_msg_to_uhf_for_downlink(
                                tcp_interface.as_mut().unwrap(),
                                deserialized_msg,
                            );
                        }
                    }
                    Err(e) => {
                        warn!("Error deserializing GS IPC msg: {:?}", e);
                        //Handle deserialization of IPC msg failure
                    }
                };
                trace!("Bulk msgs read: {}", bulk_msgs_read);
                init_ipc_gs_interface.clear_buffer();
            }
        }
        // If we are done reading bulk msgs, start protocol with GS
        if received_bulk_ack && bulk_msgs_read >= expected_msgs {
            bulk_msgs_read = 0;
            expected_msgs = 0;
            received_bulk_ack = false;
            ipc_gs_interface.as_mut().unwrap().clear_buffer();
        }

        // Poll the IPC unix domain socket for the COMS channel
        if let Some(ref mut init_ipc_coms_interface) = ipc_coms_interface {
            if init_ipc_coms_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
                trace!("Received COMS IPC Msg bytes");
                let deserialized_msg_result = deserialize_msg(&init_ipc_coms_interface.buffer);
                match deserialized_msg_result {
                    Ok(deserialized_msg) => {
                        trace!("Dserd msg body len {}", deserialized_msg.msg_body.len());
                        // Handles msg internally for COMS
                        handle_msg_for_coms(&deserialized_msg);
                    }
                    Err(e) => {
                        warn!("Error deserializing COMS IPC msg: {:?}", e);
                        //Handle deserialization of IPC msg failure
                    }
                };
                init_ipc_coms_interface.clear_buffer();
            }
        }

        // Poll the IPC unix domain socket for the UHF handler channel
        if let Some(ref mut init_ipc_uhf_interface) = ipc_uhf_interface {
            if init_ipc_uhf_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
                trace!("Received UHF Handler IPC Msg bytes");
                let deserialized_msg_result = deserialize_msg(&init_ipc_uhf_interface.buffer);
                match deserialized_msg_result {
                    Ok(deserialized_msg) => {
                        trace!("Dserd msg body len {}", deserialized_msg.msg_body.len());
                        // Handles msg internally for UHF
                        uhf_handler
                            .handle_msg_for_uhf(tcp_interface.as_mut().unwrap(), &deserialized_msg);
                    }
                    Err(e) => {
                        warn!("Error deserializing UHF Handler IPC msg: {:?}", e);
                        //Handle deserialization of IPC msg failure
                    }
                };
                init_ipc_uhf_interface.clear_buffer();
            }
        }

        let uhf_bytes_read_result = tcp_interface.as_mut().unwrap().read(&mut uhf_buf);
        match uhf_bytes_read_result {
            Ok(num_bytes_read) => {
                uhf_num_bytes_read = num_bytes_read;
            }
            Err(e) => {
                warn!("Error reading from UHF transceiver: {:?}", e);
            }
        }

        if uhf_num_bytes_read > 0 {
            trace!("Received bytes from UHF");
            let ack_msg_id = 0;
            let mut ack_msg_body = vec![0x4F, 0x4B]; // 0x4F = O , 0x4B = K  [OK
                                                     //TODO - Decrypt incomming encrypted bytes
            let decrypted_byte_result = decrypt_bytes_from_gs(&uhf_buf);
            match decrypted_byte_result {
                // After decrypting, send directly to the msg_dispatcher
                Ok(decrypted_byte_vec) => {
                    if let Some(ref mut init_ipc_cmd_interface) = ipc_cmd_interface {
                        if let Some(fd) = init_ipc_cmd_interface.data_fd.as_ref() {
                            let _ = ipc_write(fd, decrypted_byte_vec);
                        }
                    }
                }
                Err(e) => {
                    warn!("Error decrypting bytes from UHF: {:?}", e);
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
            let ack_msg = Msg::new(0, ack_msg_id, GS, COMS, 200, ack_msg_body);
            write_msg_to_uhf_for_downlink(tcp_interface.as_mut().unwrap(), ack_msg);
            uhf_buf.fill(0);
        }

        // Handle regular messages for GS
        let mut servers: Vec<&mut Option<IpcServer>> = vec![&mut gs_interface_non_bulk];
        poll_ipc_server_sockets(&mut servers);
        if gs_interface_non_bulk.as_mut().unwrap().buffer != [0u8; IPC_BUFFER_SIZE] {
            trace!(
                "GS msg server \"{}\" received data",
                gs_interface_non_bulk.as_mut().unwrap().socket_path
            );
            match deserialize_msg(&gs_interface_non_bulk.as_mut().unwrap().buffer) {
                Ok(msg) => {
                    trace!("got {:?}", msg);
                    write_msg_to_uhf_for_downlink(tcp_interface.as_mut().unwrap(), msg);
                    gs_interface_non_bulk.as_mut().unwrap().clear_buffer();
                }
                Err(err) => {
                    warn!("Error deserialising message for gs ({:?})", err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {}
