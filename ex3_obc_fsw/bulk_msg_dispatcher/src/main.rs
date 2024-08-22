use bulk_msg_slicing::*;
use common::*;
use component_ids::{DFGM, GS};
use ipc::*;
use message_structure::*;
use std::fs::{File, OpenOptions};
use std::io::Error as IoError;
use std::io::Read;
use std::thread;
use std::time::Duration;
use std::path::Path;
use std::fs;
use std::io::Write;
use std::io;
use logging::*;
use log::{debug, error, info, trace, warn};
const INTERNAL_MSG_BODY_SIZE: usize = 4089; // 4KB - 7 (header) being passed internally
fn main() -> Result<(), IoError> {
    // All connected handlers and other clients will have a socket for the server defined here
    let mut coms_interface: IpcServer = IpcServer::new("gs_bulk".to_string())?;
    let mut cmd_msg_disp_interface: IpcClient = IpcClient::new("bulk_disp".to_string())?;
    let mut messages = Vec::new();

    let log_path = "logs";
    init_logger(&log_path);

    loop {
        let coms_interface_clone = coms_interface.clone();
        let mut servers: Vec<&mut IpcServer> = vec![&mut coms_interface];
        let mut clients: Vec<&mut IpcClient> = vec![&mut cmd_msg_disp_interface];

        poll_ipc_clients(&mut clients)?;
        // Msgs from the cmd_msg_dispatcher. I.e: Commands to downlink data from a certain path.
        for client in clients {
            if let Some(msg) = handle_server_input(client)? {
                let path_bytes: Vec<u8> = msg.msg_body.clone();
                let path = get_path_from_bytes(path_bytes)?;
                match get_data_from_path(&path) {
                    Ok(bulk_msg) => {
                        trace!("Bytes expected at GS: {}", bulk_msg.msg_body.len() + 7); // +7 for header
                        // Slice bulk msg
                        // TODO - Cloning here might affect performance!!
                        messages = handle_large_msg(bulk_msg.clone(), INTERNAL_MSG_BODY_SIZE)?;

                        // Calculate num of 4KB msgs
                        let first_msg = messages[0].clone();
                        let num_of_4kb_msgs = u16::from_le_bytes([first_msg.msg_body[0],first_msg.msg_body[1]]) + 1; // account for msg containing num of msgs
                        trace!("Num of 4k msgs: {}", num_of_4kb_msgs);

                        // Start coms protocol with coms handler to downlink
                        send_num_msgs_and_bytes_to_gs(
                            num_of_4kb_msgs,
                            bulk_msg.msg_body.len() as u64,
                            coms_interface_clone.data_fd,
                        )?;
                        
                        client.clear_buffer();
                    }
                    Err(e) => {
                        warn!("Error reading data from path: {}",e);
                    }
                }
            }
        }

        poll_ipc_server_sockets(&mut servers);
        // msgs from coms_handler
        for server in servers {
            if let Some(msg) = handle_client(server)? {
                if msg.header.msg_type == MsgType::Ack as u8 {
                    // Is there a better way of differentiating between ACK's?
                    if msg.msg_body[0] == 0 {
                        for i in 0..messages.len() {
                            let serialized_msgs = serialize_msg(&messages[i])?;
                            trace!("Sending {} B", serialized_msgs.len());
                            ipc_write(coms_interface_clone.data_fd, &serialized_msgs)?;
                            trace!("Sent msg #{}", i + 1);
                            // save_data_to_file(messages[i].msg_body.clone(), 0);
                            thread::sleep(Duration::from_millis(100));
                        }
                    } else {
                        todo!()
                    }
                }
                server.clear_buffer();
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

/// Same as handle client but for getting a msg from the cmd_msg_disp
fn handle_server_input(client: &IpcClient) -> Result<Option<Msg>, IoError> {
    if client.buffer != [0u8; IPC_BUFFER_SIZE] {
        trace!(
            "Server {} received data",
            client.socket_path
        );
        //Build Msg from received bytes and get body which contains path
        Ok(Some(deserialize_msg(&client.buffer)?))
    } else {
        Ok(None)
    }
}

/// This is the communication protocol that will execute each time the Bulk Msg Dispatcher wants
/// to send a Bulk Msg to the coms handler for downlinking.
fn send_num_msgs_and_bytes_to_gs(num_msgs: u16, num_bytes: u64, fd: Option<i32>) -> Result<(), IoError> {
    // 1. Send Msg to coms handler indicating Bulk Msg and buffer size needed
    let mut num_msgs_bytes: Vec<u8> = num_msgs.to_le_bytes().to_vec();
    let mut num_bytes_bytes: Vec<u8> = num_bytes.to_le_bytes().to_vec();
    num_msgs_bytes.append(&mut num_bytes_bytes);
    let num_msg: Msg = Msg::new(MsgType::Bulk as u8, GS, DFGM, 2, 0,  num_msgs_bytes);
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
        src_id = DFGM;
    } else if path.contains("iris") {
        src_id = component_ids::ComponentIds::IRIS as u8;
    } else if path.contains("coms") {
        src_id = component_ids::ComponentIds::COMS as u8;
    }

    // Create the Msg object
    let bulk_msg: Msg = Msg::new(MsgType::Bulk as u8, 0, 7, src_id, 0, data);
    Ok(bulk_msg)
}

// TMP Stolen from gs_cli. Used for testing to run diffs between
// Files before and after they're downlinked
fn save_data_to_file(data: Vec<u8>, src: u8) -> std::io::Result<()> {
    // ADD future dir names here depending on source
    let dir_name = if src == DFGM {
        "dfgm"
    } else if src == 99 {
        "test"
    } else {
        "misc"
    };

    fs::create_dir_all(dir_name)?;
    let mut file_path = Path::new(dir_name).join("data");

    // Append number to file name if it already exists
    let mut count = 0;
    while file_path.exists() {
        count += 1;
        file_path = Path::new(dir_name).join(format!("data{}", count));
    }
    let mut file = File::create(file_path)?;

    file.write_all(&data)?;

    Ok(())
}
