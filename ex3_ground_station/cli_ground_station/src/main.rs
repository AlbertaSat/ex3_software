/*
Written by Devin Headrick and Rowan Rasmusson
Summer 2024


References:
    - CLI book: https://rust-cli.github.io/book/index.html

TODO
    - Test various operator inputs and edge cases
    - Have the 'up' key bring back the previously entered command

*/

use bulk_msg_slicing::*;
use common::*;
use common::ports::SIM_COMMS_PORT;
use libc::c_int;
use message_structure::*;
use std::fs::File;
use std::path::Path;
use tcp_interface::*;

use libc::{poll, POLLIN};
use std::fs;
use std::io::prelude::*;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio;
use tokio::sync::Mutex;

const WAIT_FOR_ACK_TIMEOUT: u64 = 10; // seconds a receiver (GS or SC) will wait before timing out and asking for a resend
const STDIN_POLL_TIMEOUT: c_int = 10;

//TOOD - create a new file for each time the program is run
//TODO - get file if one already this time the 'program is run' - then properly append JSON data (right now it just appends json data entirely)
//TODO - get the current users name
//TODO - Store the associated build msg with the operator entered string (if the msg is built successfully)
/// Store string entered by operator in a JSON file, with other metadata like timestamp, operator name, TBD ...
// fn store_operator_entered_string(operator_str: String) {
//     // Write the operator entered string to a file using JSON, with a time stamp
//     let utc: DateTime<Utc> = Utc::now();
//     let operator_json = json!({
//         "time": utc.to_string(),
//         "operator_input": operator_str,
//         "user: " : "Default Operator"
//     });
//     let file = std::fs::OpenOptions::new()
//         .write(true)
//         .append(true)
//         .create(true)
//         .open("operator_input.json")
//         .unwrap();

//     let mut writer = BufWriter::new(&file);
//     serde_json::to_writer(&mut writer, &operator_json).unwrap();
//     let _ = writer.flush();
// }

/// Build a message from operator input string, where values are delimited by spaces.
/// 1st value is the destination component string - converted to equivalent component id.
/// 2nd value is opcode num - converted from ascii to a byte.
/// Remaining values are the data - converted from ascii into bytes.
fn build_msg_from_operator_input(operator_str: String) -> Result<Msg, std::io::Error> {
    //Parse input string by spaces
    let operator_str_split: Vec<&str> = operator_str.split(" ").collect();

    if operator_str_split.len() < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Not enough arguments",
        ));
    }

    let dest_id = component_ids::ComponentIds::from_str(operator_str_split[0]).unwrap() as u8;
    let mut msg_body: Vec<u8> = Vec::new();
    let mut msg_type = 0;
    let mut opcode = 0;

    // This is for the Bulk Msg Disp to parse and determine the path it needs to use to get the data
    if dest_id == component_ids::ComponentIds::BulkMsgDispatcher as u8 {
        msg_type = MsgType::Cmd as u8;
        msg_body = operator_str_split[1].as_bytes().to_vec();
    } else {
        opcode = operator_str_split[1].parse::<u8>().unwrap();

        for data_byte in operator_str_split[2..].into_iter() {
        msg_body.push(data_byte.parse::<u8>().unwrap());
        }
    }
    
    let msg = Msg::new(msg_type, 0, dest_id, component_ids::ComponentIds::GS as u8, opcode, msg_body);
    println!("Built msg: {:?}", msg);
    Ok(msg)
}

/// Takes mutable reference to the awaiting ack flag, derefs it and sets the value
fn handle_ack(msg: Msg, awaiting_ack: &mut bool) -> Result<(), std::io::Error> {
    //TODO - handle if the Ack is OK or ERR , OR not an ACK at all
    println!("Received ACK: {:?}", msg);
    *awaiting_ack = false;
    Ok(())
}

fn send_msg_to_sc(msg: Msg, tcp_interface: &mut TcpInterface) {
    let serialized_msg = serialize_msg(&msg).unwrap();
    let ret = tcp_interface.send(&serialized_msg).unwrap();
    println!("Sent {} bytes to Coms handler", ret);
    std::io::stdout().flush().unwrap();
}

/// Sleep for 1 second intervals - and check if the await ack flag has been reset each second
/// This is so that if an ACK is read, then this task ends
async fn awaiting_ack_timeout_task(awaiting_ack_clone: Arc<Mutex<bool>>) {
    let mut count = 0;
    while count < WAIT_FOR_ACK_TIMEOUT {
        tokio::time::sleep(Duration::from_secs(1)).await;
        count += 1;
        let lock = awaiting_ack_clone.lock().await;
        if *lock == false {
            return;
        }
    }
    let mut lock = awaiting_ack_clone.lock().await;
    *lock = false;
    println!("WARNING: NO ACK received - Last sent message may not have been received by SC.");
}
/// Function to represent the state of reading bulk msgs continuously.
/// It modifies the bulk_messages in place by taking a mutable reference.
fn read_bulk_msgs(
    tcp_interface: &mut TcpInterface,
    bulk_messages: &mut Vec<Msg>,
    num_msgs_to_recv: u16,
) -> Result<(), std::io::Error> {
    let mut bulk_buf = [0u8; 4096];
    let mut num_msgs_recvd = 0;
    println!("Num msgs incoming: {}", num_msgs_to_recv);
    while num_msgs_recvd < num_msgs_to_recv {
        let bytes_read = tcp_interface.read(&mut bulk_buf)?;
        if bytes_read > 0 {
            let cur_msg = deserialize_msg(&bulk_buf[0..bytes_read])?;
            if cur_msg.header.msg_type == MsgType::Bulk as u8 {
                let seq_id = u16::from_le_bytes([cur_msg.msg_body[0], cur_msg.msg_body[1]]);
                println!("Received msg #{}", seq_id);
                // println!("{:?}", cur_msg);
                bulk_messages.push(cur_msg.clone());
                thread::sleep(Duration::from_millis(10));
                num_msgs_recvd += 1;
            }
        }
    }

    Ok(())
}


/// Function to save downlinked data to a file
fn save_data_to_file(data: Vec<u8>, src: u8) -> std::io::Result<()> {
    let src_comp_enum = component_ids::ComponentIds::from(src);
    // ADD future dir names here depending on source
    let mut dir_name: String = match src_comp_enum {
        component_ids::ComponentIds::DFGM => "dfgm".to_string(),
        component_ids::ComponentIds::IRIS => "iris".to_string(),
        component_ids::ComponentIds::COMS => "coms".to_string(),
        component_ids::ComponentIds::DUMMY => "dummy".to_string(),
        _ => "misc".to_string()
    };

    // Prepend directory we want it to be created in
    dir_name.insert_str(0, "ex3_ground_station/");
    fs::create_dir_all(dir_name.clone())?;
    let mut file_path = Path::new(&dir_name).join("data");

    // Append number to file name if it already exists
    let mut count = 0;
    while file_path.exists() {
        count += 1;
        file_path = Path::new(&dir_name).join(format!("data{}", count));
    }
    let mut file = File::create(file_path)?;

    file.write_all(&data)?;

    Ok(())
}


/// Function for rebuilding msgs that have been downlinked from the SC
/// First, it takes a chunk of 128B msgs and makes a 4KB packet out of that
/// Then, takes the vector of 4KB packets and makes one large msg using it
fn process_bulk_messages(
    bulk_messages: Vec<Msg>,
    num_bytes: usize,
) -> Result<Msg, &'static str> {
    let mut reconstructed_large_msg = reconstruct_msg(bulk_messages)?;
    reconstructed_large_msg.msg_body = reconstructed_large_msg.msg_body[0..num_bytes].to_vec();
    Ok(reconstructed_large_msg)
}

#[tokio::main]
async fn main() {
    println!("Beginning CLI Ground Station...");
    println!("Waiting for connection to Coms handler via TCP...");
    let mut tcp_interface =
        TcpInterface::new_server("127.0.0.1".to_string(), SIM_COMMS_PORT).unwrap();
    println!("Connected to Coms handler via TCP ");

    let mut bulk_messages: Vec<Msg> = Vec::new();
    let stdin_fd = std::io::stdin().as_raw_fd();

    loop {
        let mut fds = [libc::pollfd {
            fd: stdin_fd,
            events: POLLIN as i16,
            revents: 0,
        }];

        // Poll stdin for input
        let ret = unsafe { poll(fds.as_mut_ptr(), 1, STDIN_POLL_TIMEOUT) }; // 10 ms timeout
        if ret > 0 && fds[0].revents & POLLIN as i16 != 0 {
            let mut input = String::new();
            let mut stdin = std::io::stdin().lock();
            stdin.read_line(&mut input).unwrap();
            let input = input.trim().to_string();

            let msg_build_res = build_msg_from_operator_input(input);

            match msg_build_res {
                Ok(msg) => {
                    send_msg_to_sc(msg, &mut tcp_interface);

                    let mut buf = [0u8; 128];
                    let awaiting_ack = Arc::new(Mutex::new(true));
                    let awaiting_ack_clone = Arc::clone(&awaiting_ack);

                    tokio::task::spawn(async move {
                        awaiting_ack_timeout_task(awaiting_ack_clone).await;
                    });

                    while *awaiting_ack.lock().await == true {
                        let bytes_read = tcp_interface.read(&mut buf).unwrap();
                        if bytes_read > 0 {
                            let recvd_msg = deserialize_msg(&buf).unwrap();
                            if recvd_msg.header.op_code == 200 {
                                let _ = handle_ack(recvd_msg, &mut *awaiting_ack.lock().await);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error building message: {}", e);
                }
            }
        } else {
            let mut read_buf = [0; 128];

            let bytes_received = match tcp_interface.read(&mut read_buf) {
                Ok(len) => len,
                Err(e) => {
                    println!("read failed: {e}");
                    break;
                }
            };
            if bytes_received > 0 {
                let recvd_msg = deserialize_msg(&read_buf).unwrap();
                // Bulk Msg Downlink Mode. Will stay in this mode until all packets are received (as of now).
                if recvd_msg.header.msg_type == MsgType::Bulk as u8 {
                    let num_msgs_to_recv =
                        u16::from_le_bytes([recvd_msg.msg_body[0], recvd_msg.msg_body[1]]);
                    let bytes = [recvd_msg.msg_body[2],
                        recvd_msg.msg_body[3],
                        recvd_msg.msg_body[4],
                        recvd_msg.msg_body[5],
                        recvd_msg.msg_body[6],
                        recvd_msg.msg_body[7],
                        recvd_msg.msg_body[8],
                        recvd_msg.msg_body[9]];
                    let num_bytes_to_recv = u64::from_le_bytes(bytes);
                    // build_and_send_ack(
                    //     &mut tcp_interface,
                    //     recvd_msg.header.msg_id.clone(),
                    //     recvd_msg.header.source_id,
                    //     recvd_msg.header.dest_id.clone(),
                    // );
                    // Listening mode for bulk msgs
                    read_bulk_msgs(
                        &mut tcp_interface,
                        &mut bulk_messages,
                        num_msgs_to_recv,
                    )
                    .unwrap();
                    // clone bulk_messages BUT maybe hurts performance if there's tons of packets
                    match process_bulk_messages(bulk_messages.clone(), num_bytes_to_recv as usize) {
                        Ok(large_msg) => {
                            println!("Successfully reconstructed 4K messages");
                            match save_data_to_file(
                                large_msg.msg_body,
                                large_msg.header.source_id,
                            ) {
                                Ok(_) => {
                                    println!("Data saved to file");
                                }
                                Err(e) => {
                                    eprintln!("Error writing data to file: {}", e);
                                }
                            }
                        }
                        Err(e) => eprintln!("Error reconstructing 4K messages: {}", e),
                    }

                    println!(
                        "We have {} bulk msgs including initial header msg",
                        bulk_messages.len()
                    );
                    
                }
                println!("Received Data: {:?}", read_buf);
            } else {
                // Deallocate memory of these messages. Reconstructed version 
                // has been written to a file. This is slightly slower than .clear() though
                bulk_messages = Vec::new();
            }
        }
    }
}
