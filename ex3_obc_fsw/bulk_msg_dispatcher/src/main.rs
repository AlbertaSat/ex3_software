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
const INTERNAL_MSG_BODY_SIZE: usize = 4091; // 4KB - 5 (header) being passed internally
fn main() -> Result<(), IoError> {
    // All connected handlers and other clients will have a socket for the server defined here
    let mut dfgm_interface: IpcServer = IpcServer::new("dfgm_bulk".to_string())?;
    let mut gs_interface: IpcServer = IpcServer::new("gs_bulk".to_string())?;
    let mut messages = Vec::new();

    loop {
        let gs_interface_clone = gs_interface.clone();
        let mut servers: Vec<&mut IpcServer> = vec![&mut dfgm_interface, &mut gs_interface];
        poll_ipc_server_sockets(&mut servers);

        for server in servers {
            if let Some(msg) = handle_client(server)? {
                if msg.header.msg_type == MsgType::Bulk as u8 {
                    let path_bytes: Vec<u8> = msg.msg_body.clone();
                    let path = get_path_from_bytes(path_bytes)?;
                    let bulk_msg = get_data_from_path(&path)?;
                    println!("Bytes expected at GS: {}", bulk_msg.msg_body.len() + 5); // +5 for header
                                                                                       // Slice bulk msg
                    messages = handle_large_msg(bulk_msg, INTERNAL_MSG_BODY_SIZE)?;

                    // Start coms protocol with GS handler to downlink
                    send_num_msgs_to_gs(
                        u16::from_le_bytes([messages[0].msg_body[0], messages[0].msg_body[1]]),
                        gs_interface_clone.data_fd,
                    )?;

                    // 2. Wait for ACK from GS handler
                    // 3. Send Msg's contained in messages
                    // 4. Wait for ACK from GS it got all the messages
                } else if msg.header.msg_type == MsgType::Ack as u8 {
                    // Is there a better way of differentiating between ACK's?
                    if msg.msg_body[0] == 0 {
                        for i in 0..messages.len() {
                            let serialized_msgs = serialize_msg(&messages[i])?;
                            println!("Sending {} B", serialized_msgs.len());
                            ipc_write(gs_interface_clone.data_fd, &serialized_msgs)?;
                            println!("Sent msg #{}", i + 1);
                            save_data_to_file(messages[i].msg_body.clone(), 0);
                            thread::sleep(Duration::from_millis(500));
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
    println!("Got path: {}", path);
    Ok(path)
}

/// In charge of getting the file path from a Msg sent to the Bulk dispatcher from a handler
fn handle_client(server: &IpcServer) -> Result<Option<Msg>, IoError> {
    if server.buffer != [0u8; IPC_BUFFER_SIZE] {
        println!(
            "Server {} received data: {:?}",
            server.socket_path, &server.buffer
        );
        //Build Msg from received bytes and get body which contains path
        Ok(Some(deserialize_msg(&server.buffer)?))
    } else {
        Ok(None)
    }
}

/// This is the communication protocol that will execute each time the Bulk Msg Dispatcher wants
/// to send a Bulk Msg to the GS handler for downlinking.
fn send_num_msgs_to_gs(num_msgs: u16, fd: Option<i32>) -> Result<(), IoError> {
    // 1. Send Msg to GS handler indicating Bulk Msg and buffer size needed
    let num_bytes: Vec<u8> = num_msgs.to_le_bytes().to_vec();
    let num_msg: Msg = Msg::new(MsgType::Bulk as u8, GS, DFGM, 2, 0, num_bytes);
    ipc_write(fd, &serialize_msg(&num_msg)?)?;
    Ok(())
}

/// This function will take all current data that is stored in a provided path and
/// append it to the body of a bulk Msg. This Msg will then be sliced.
fn get_data_from_path(path: &str) -> Result<Msg, std::io::Error> {
    let mut file: File = OpenOptions::new()
        .read(true)
        .open(format!("{}/data", path))?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;
    let bulk_msg: Msg = Msg::new(MsgType::Bulk as u8, 0, 7, 3, 0, data);
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
