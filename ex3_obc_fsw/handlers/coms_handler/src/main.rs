/*
Written By Devin Headrick
Summer 2024

For tall thin all we want this to do is talk to this (via TCP) and have it relay its data to the message dispatcher (via IPC unix domain socket)

TODO - Detect if connection to either msg dispatcher or UHF transceiver is lost, and handle that - attempt to reconnect
TODO - implement a 'gs' connection flag, which the handler uses to determine whether or not it can downlink messages to the ground station.
TODO - mucho error handling
*/
use log::{debug, trace, warn};
use common::logging::*;

use common::component_ids::ComponentIds;
use common::constants::UHF_MAX_MESSAGE_SIZE_BYTES;
use common::opcodes;
use common::ports;
use interface::{ipc::*, tcp::*, Interface};
use common::message_structure::{SerializeAndDeserialize,
                                deserialize_msg, serialize_msg,
                                AckCode, CmdMsg, Msg, MsgType};
use std::vec;
mod uhf_handler;
use uhf_handler::UHFHandler;

/// Setup function for decrypting incoming messages from the UHF transceiver
/// This just decrypts the bytes and does not return a message from the bytes
fn decrypt_bytes_from_gs(encrypted_bytes: &[u8]) -> Result<&[u8], std::io::Error> {
    // TODO - Decrypt the message
    let decrypted_byte_vec = encrypted_bytes;
    Ok(decrypted_byte_vec)
}

/// For messages directed FOR the coms handler directly. Based on the opcode of the message, perform some action
fn handle_msg_for_coms(msg: &Msg) {
    let opcode_enum = opcodes::COMS::from(msg.header.op_code);
    match opcode_enum {
        opcodes::COMS::GetHK => {
            trace!("Opcode 3: Get House Keeping Data from COMS Handler for UHF");
        }
        _ => debug!("Invalid msg opcode"),
    }
}

/// Function to send the initial messages containing num of 4KB msgs to expect and the number of
/// data bytes to expect once the msg is rebuilt
fn send_initial_bulk_to_gs(initial_msg: Msg, interface: &mut TcpInterface) {
    write_msg_to_uhf_for_downlink(interface, initial_msg);
}

/// Function for sending an ACK to the bulk disp letting it know to send bulk msgs for downlink
fn send_bulk_ack(iface: &mut IpcClient) -> Result<(), std::io::Error> {
    let ack_msg = Msg::new(
        MsgType::Ack as u8,
        20,
        ComponentIds::BulkMsgDispatcher as u8,
        ComponentIds::COMS as u8,
        0,
        vec![0],
    );
    iface.send(&serialize_msg(&ack_msg)?)?;
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
    let ipaddr = std::env::args().nth(1).unwrap_or("localhost".to_string());
    let log_path = "ex3_obc_fsw/handlers/coms_handler/logs";
    init_logger(log_path);
    trace!("Logger initialized");
    trace!("Beginning Coms Handler on {ipaddr}:{}", ports::SIM_ESAT_UART_PORT);

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
    let ipc_cmd_interface_res = IpcClient::new("cmd_dispatcher".to_string());
    let mut ipc_cmd_interface = match ipc_cmd_interface_res {
        Ok(i) => Some(i),
        Err(e) => {
            warn!("Cannot create COMS pipeline: {e}");
            None
        }
    };

    // This is the client that listens for bulk messages to be transmit to the groundstaion
    let ipc_gs_interfac_res = IpcClient::new("gs_bulk".to_string());
    let mut bulk_downlink_interface = match ipc_gs_interfac_res {
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
        match TcpInterface::new_client(ipaddr, ports::SIM_ESAT_UART_PORT) {
            Ok(tcp) => Some(tcp),
            Err(e) => {
                warn!("Error creating UHF interface: {e}");
                None
            }
        };
    // Temporary fix, this should be in a poll with all the other servers
    // This sets the read to be completely unblocking, so it comes with quite a bit of function
    // call overhead.
    let _ = tcp_interface.as_ref().as_mut().unwrap().stream.set_read_timeout(Some(std::time::Duration::from_millis(50)));
    let mut uhf_buf = vec![0; UHF_MAX_MESSAGE_SIZE_BYTES]; //Buffer to read incoming messages from UHF
    let mut uhf_num_bytes_read = 0;
    let mut received_bulk_ack = false;
    let mut bulk_msgs_read = 0;
    let mut expected_msgs = 0;

    loop {
        uhf_buf.fill(0);
        // Poll both the UHF transceiver and IPC unix domain socket for the GS channel
        let mut clients = vec![
            &mut bulk_downlink_interface,
            &mut ipc_cmd_interface,
        ];
        let mut servers = vec![
            &mut ipc_coms_interface,
            &mut ipc_uhf_interface,
        ];
        let _ = poll_ipc_server_sockets(&mut servers);
        let ipc_bytes_read_res = poll_ipc_clients(&mut clients);

        if let Some(ref mut init_ipc_gs_interface) = bulk_downlink_interface {
            if init_ipc_gs_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
                trace!("Received IPC Msg bytes for GS");
                let deserialized_msg_result = deserialize_msg(&init_ipc_gs_interface.buffer);
                match deserialized_msg_result {
                    Ok(deserialized_msg) => {
                        // if the msg bulk type and we have not send the bulk ack then send it to 
                        // bulk msg dispatcher.
                        if deserialized_msg.header.msg_type == MsgType::Bulk as u8
                            && !received_bulk_ack
                        {
                            trace!("Sending ACK to bulk dispatcher, should be sending messages now");
                            if let Some(e) = send_bulk_ack(init_ipc_gs_interface).err() {
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
                            // Here where we read incoming bulk msgs from bulk_msg_disp
                            if bulk_msgs_read < expected_msgs {
                                if let Ok((ipc_bytes_read, _ipc_name)) = ipc_bytes_read_res {
                                    if let Some(client_addr) = init_ipc_gs_interface.server_addr {
                                        if client_addr.path().unwrap().to_str().unwrap().contains("gs") {
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
            trace!("Bulk downlink completed... Restarting bulk state machine.");
            bulk_msgs_read = 0;
            expected_msgs = 0;
            received_bulk_ack = false;
            bulk_downlink_interface.as_mut().unwrap().clear_buffer();
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
            },
            // Part of the temporary blocking fix.
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                uhf_num_bytes_read = 0;
            },
            Err(e) => {
                warn!("Error reading from UHF transceiver: {:?}", e);
            }
        }
        if uhf_num_bytes_read > 0 {
            trace!("Received bytes from UHF");
            //TODO - Decrypt incomming encrypted bytes
            let decrypted_byte_result = decrypt_bytes_from_gs(&uhf_buf);
            let mut status = AckCode::Success;
            let mut ackbody = "Error: ".to_string();
            let cmd: CmdMsg = match decrypted_byte_result {
                // After decrypting, send directly to the msg_dispatcher
                Ok(msg) => {
                    if let Some(ref mut init_ipc_cmd_interface) = ipc_cmd_interface {
                        if let Some(_server_addr) = init_ipc_cmd_interface.server_addr {
                            match init_ipc_cmd_interface.send(msg) {
                                Ok(len) => debug!("coms: forwarded {} bytes", len),
                                Err(e) => {
                                    status = AckCode::Failed;
                                    ackbody.push_str(&format!("write to cmd dispatcher failed - {}", e));
                                }
                            };
                        }
                    }
                    else {
                        status = AckCode::Failed;
                        ackbody.push_str("no connection to cmd dispatcher");
                    }
                    CmdMsg::deserialize_from_bytes(msg)
                }
                Err(e) => {
                    status = AckCode::Failed;
                    ackbody.push_str(&format!("decryption failed - {}", e));
                    CmdMsg::deserialize_from_bytes(&uhf_buf)
                }
            };

            if status == AckCode::Failed {
                warn!("{}", ackbody);
                /* Nack failed messages back to the sender */
                let nack = Msg::new(MsgType::Ack as u8, cmd.header.msg_id,
                                    ComponentIds::GS as u8, ComponentIds::COMS as u8,
                                    status as u8, ackbody.as_bytes().to_vec());
                write_msg_to_uhf_for_downlink(tcp_interface.as_mut().unwrap(), nack);
                uhf_buf.fill(0);
            }
            else {
                // send ack to groundstation if we recv command successfully
                // not sure what an actual ack would look like, we probably want to include some
                // information in the ack body, for now it is blank
                let ackbody = [];
                let ack = Msg::new(MsgType::Ack as u8, cmd.header.msg_id,
                                    ComponentIds::GS as u8, ComponentIds::COMS as u8,
                                    status as u8, ackbody.to_vec());
                write_msg_to_uhf_for_downlink(tcp_interface.as_mut().unwrap(), ack);
                uhf_buf.fill(0);
            }
        }

        let mut servers: Vec<&mut Option<IpcServer>> = vec![&mut gs_interface_non_bulk];
        let _ = poll_ipc_server_sockets(&mut servers);
        // Handle regular messages for GS
        if let Some(ref mut gs_if) = gs_interface_non_bulk {
            if gs_if.buffer != [0u8; IPC_BUFFER_SIZE] {
                trace!("GS msg server \"{}\" received data", gs_if.socket_path);
                match deserialize_msg(&gs_if.buffer) {
                    Ok(msg) => {
                        trace!("got {:?}", msg);
                        write_msg_to_uhf_for_downlink(tcp_interface.as_mut().unwrap(), msg);
                        gs_if.clear_buffer();
                    }
                    Err(err) => {
                        warn!("Error deserialising message for gs ({:?})", err);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {}
