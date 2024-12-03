use common::bulk_msg_slicing::*;
use common::*;
use interface::ipc::*;
use message_structure::*;
use std::fs::{File, OpenOptions};
use std::io::Error as IoError;
use std::io::Read;
use std::os::fd::OwnedFd;
use std::thread;
use std::time::Duration;
use std::path::Path;
use std::{fs, io};
use logging::*;
use log::{trace, warn};

const INTERNAL_MSG_BODY_SIZE: usize = 4088; // 4KB - 8 (header) being passed internally
fn main() -> Result<(), IoError> {
    // All connected handlers and other clients will have a socket for the server defined here
    // This pipeline is directly to the coms_handler to be directly downlinked sliced data packets
    let coms_interface_res = IpcServer::new("gs_bulk".to_string());
    let mut coms_interface = match coms_interface_res {
        Ok(s) => Some(s),
        Err(e) => {
            warn!("Connot create bulk to ground pipeline: {e}");
            None
        }
    };

    // This interface is for recieving commands from the cmd_dispatcher telling this bulk_msg_dispatcher what to slice and pass on
    let cmd_disp_interface_res = IpcServer::new("BulkMsgDispatcher".to_string());
    let mut cmd_disp_interface = match cmd_disp_interface_res {
        Ok(s) => Some(s),
        Err(e) => {
            warn!("Connot create cmd_disp to bulk_disp pipeline: {e}");
            None
        }
    };

    let mut messages = Vec::new();
    let mut num_of_4kb_msgs = 1;
    let mut num_bytes = 4098;

    let log_path = "logs";
    init_logger(log_path);

    loop {
        let mut servers = vec![&mut coms_interface, &mut cmd_disp_interface];
        thread::sleep(Duration::from_secs(1));
        poll_ipc_server_sockets(&mut servers);
    
        for server in servers.into_iter().flatten() {
            if let Some(msg) = handle_client(server)? {
                if server.socket_path.contains("gs_bulk") {
                    if msg.header.msg_type == MsgType::Ack as u8 {
                        // TODO: Do we need the msg body to be 0? Will this disp see any other types of ACKs?
                        if msg.msg_body[0] == 0 {
                            for (i, message) in messages.iter().enumerate() {
                                let serialized_msg = serialize_msg(message)?;
                                trace!("Sending {} B", serialized_msg.len());
                                if let Some(data_fd) = &server.data_fd {
                                    ipc_write(data_fd, &serialized_msg)?;
                                } else {
                                    warn!("No data file descriptor found in coms_interface.");
                                    break;
                                }
                                trace!("Sent msg #{}", i + 1);
                                thread::sleep(Duration::from_micros(1));
                            }
                            messages.clear();
                            server.clear_buffer();
                        } else {
                            todo!();
                        }
                    }
                } else if server.socket_path.contains("BulkMsgDispatcher") {
                    let path_bytes: Vec<u8> = msg.msg_body.clone();
                    let path = get_path_from_bytes(path_bytes)?;
                    match get_data_from_path(&path) {
                        Ok(bulk_msg) => {
                            trace!("Bytes expected at GS: {}", bulk_msg.msg_body.len() + HEADER_SIZE); // +8 for header
                            messages = handle_large_msg(bulk_msg.clone(), INTERNAL_MSG_BODY_SIZE)?;
    
                            let first_msg = messages[0].clone();
                            num_of_4kb_msgs = u16::from_le_bytes([first_msg.msg_body[0], first_msg.msg_body[1]]) + 1;
                            num_bytes = bulk_msg.msg_body.len() as u64;
                            trace!("Num of 4k msgs: {}", num_of_4kb_msgs);
    
                            server.clear_buffer();
                        }
                        Err(e) => {
                            warn!("Error reading data from path: {}", e);
                        }
                    }
                }
            }
        }
    // Separate block for sending data if messages are available
    if !messages.is_empty() {
        if let Some(ref mut gs_bulk_server) = coms_interface {
            if let Some(data_fd) = &gs_bulk_server.data_fd {
                send_num_msgs_and_bytes_to_gs(num_of_4kb_msgs, num_bytes, data_fd)?;
            } else {
                warn!("No data file descriptor found in coms_interface.");
            }
            gs_bulk_server.clear_buffer();
        }
    }
    }
    
}

fn get_path_from_bytes(path_bytes: Vec<u8>) -> Result<String, IoError> {
    let mut path: String = String::from_utf8(path_bytes).expect("Found invalid UTF-8 in path.");
    path = path.trim_matches(char::from(0)).to_string();
    trace!("Got path: {}", path);
    Ok(path)
}

/// In charge of getting the file path from a Msg sent to the Bulk dispatcher from a handler
fn handle_client(server: &IpcServer) -> Result<Option<Msg>, IoError> {
    if server.buffer != [0u8; IPC_BUFFER_SIZE] {
        trace!(
            "Server {} received data",
            server.socket_path
        );
        //Build Msg from received bytes and get body which contains path
        Ok(Some(deserialize_msg(&server.buffer)?))
    } else {
        Ok(None)
    }
}

/// This is the communication protocol that will execute each time the Bulk Msg Dispatcher wants
/// to send a Bulk Msg to the coms handler for downlinking.
fn send_num_msgs_and_bytes_to_gs(num_msgs: u16, num_bytes: u64, fd: &OwnedFd) -> Result<(), IoError> {
    // 1. Send Msg to coms handler indicating Bulk Msg and buffer size needed
    let mut num_msgs_bytes: Vec<u8> = num_msgs.to_le_bytes().to_vec();
    let mut num_bytes_bytes: Vec<u8> = num_bytes.to_le_bytes().to_vec();
    num_msgs_bytes.append(&mut num_bytes_bytes);
    let num_msg: Msg = Msg::new(MsgType::Bulk as u8, 0,
                                ComponentIds::GS as u8, ComponentIds::DFGM as u8,
                                2, num_msgs_bytes);
    ipc_write(fd, &serialize_msg(&num_msg)?)?;
    Ok(())
}

/// This function will take all current data that is stored in a provided path and
/// append it to the body of a bulk Msg. This Msg will then be sliced.
fn get_data_from_path(path: &str) -> Result<Msg, std::io::Error> {
    let dir_path = Path::new(path);

    // Get the first file in the directory
    let file_name = match fs::read_dir(dir_path)?
        .filter_map(Result::ok)
        .find(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
    {
        Some(entry) => entry.path(),
        None => return Err(io::Error::new(io::ErrorKind::NotFound, "No files found in the directory")),
    };

    // Open the file
    let mut file: File = OpenOptions::new()
        .read(true)
        .open(file_name)?;

    // Read the file content into a vector
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;

    // Get src id
    let mut src_id: u8 = 0;
    if path.contains("dfgm") {
        src_id = ComponentIds::DFGM as u8;
    } else if path.contains("iris") {
        src_id = component_ids::ComponentIds::IRIS as u8;
    } else if path.contains("coms") {
        src_id = component_ids::ComponentIds::COMS as u8;
    }

    // Create the Msg object
    let bulk_msg: Msg = Msg::new(MsgType::Bulk as u8, 0, 7, src_id, 0, data);
    Ok(bulk_msg)
}
