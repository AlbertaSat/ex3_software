/*
Written by Devin Headrick and Rowan Rasmusson
Summer 2024


References:
    - CLI book: https://rust-cli.github.io/book/index.html

TODO
    - Test various operator inputs and edge cases
    - Have the 'up' key bring back the previously entered command

*/
mod bulk;
mod eps;
mod shell;

use common::bulk_msg_slicing::*;
use common::{ports, ComponentIds};
use common::message_structure::*;
use libc::c_int;
use std::fs::File;
use std::path::Path;
use std::str::from_utf8;
use interface::{tcp::*, Interface};

use libc::{poll, POLLIN};
use std::io::prelude::*;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::{fs, process};
use std::time::Duration;
use strum::IntoEnumIterator;
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

fn help() {
    println!("Available commands:");
    println!("<payload> <args>, where <payload> is:");
    for x in ComponentIds::iter() {
        println!("  {}", x);
    }
    println!("quit/exit");
    println!("help/?");
}

/// Build a message from operator input string, where values are delimited by spaces.
/// 1st value is the destination component string - converted to equivalent component id.
/// 2nd value is opcode num - converted from ascii to a byte.
/// Remaining values are the data - converted from ascii into bytes.
fn build_msg_from_operator_input(operator_str: String) -> Option<Msg> {
    //Parse input string by spaces
    let input_tokens: Vec<&str> = operator_str.split(" ").collect();

    if input_tokens.len() < 2 {
        let cmd = input_tokens[0];
        if cmd == "exit" || cmd == "quit" {
            process::exit(0);  // does not return
        }
        else {
            help();
        }
        return None;
    }

    let cmd = input_tokens[0].to_uppercase();
    let payload = match ComponentIds::iter().find(|x| cmd == format!("{x}")) {
        Some(p) => p,
        None => {
            let p = ComponentIds::BulkMsgDispatcher;
            if input_tokens[0] == format!("{p}") {
                p
            }
            else {
                println!("Unknown payload: {}", input_tokens[0]);
                return None;
            }
        }
    };

    let mut opcode = 0;
    let msg_type = MsgType::Cmd as u8;

    let msg_body = match payload {
        ComponentIds::BulkMsgDispatcher => bulk::parse_cmd(&input_tokens[1..]),
        ComponentIds::EPS => eps::parse_cmd(&input_tokens[1..]),
        ComponentIds::SHELL => shell::parse_cmd(&input_tokens[1..]),
        _ => {
            opcode = input_tokens[1].parse::<u8>().unwrap();
            Some(input_tokens[2..].join(" ").as_bytes().to_vec())
        }
    };

    match msg_body {
        Some(b) => {
            let msg = Msg::new(msg_type, 0, payload as u8, ComponentIds::GS as u8, opcode, b);
            println!("Built msg: {:?}", msg);
            Some(msg)
        },
        None => None,
    }
}

/// Takes mutable reference to the awaiting ack flag, derefs it and sets the value
fn handle_response(msg: &Msg) {
    //TODO - handle if the Ack is OK or ERR , OR not an ACK at all
    if msg.header.op_code == AckCode::Failed as u8 {
        match std::str::from_utf8(&msg.msg_body) {
            Ok(s) => println!("Command failed: {}", s),
            Err(e) => println!("Command failed, respnse corrupt: {}", e),
        }
    }
    else {
        if let Ok(payload) = ComponentIds::try_from(msg.header.source_id) {
            println!("got response from {}", payload);
        }
    };
}

fn send_msg_to_sc(msg: Msg, tcp_interface: &mut TcpInterface) {
    let serialized_msg = serialize_msg(&msg).unwrap();
    match tcp_interface.send(&serialized_msg) {
        Ok(len) => println!("Sent {} bytes to Coms handler", len),
        Err(e) => println!("Send to COMs handler failed: {}", e),
    };
}

/// Sleep for 1 second intervals - and check if the await ack flag has been reset each second
/// This is so that if an ACK is read, then this task ends
async fn awaiting_ack_timeout_task(awaiting_ack_clone: Arc<Mutex<bool>>) {
    let mut count = 0;
    while count < WAIT_FOR_ACK_TIMEOUT {
        tokio::time::sleep(Duration::from_secs(1)).await;
        count += 1;
        let lock = awaiting_ack_clone.lock().await;
        if !(*lock) {
            return;
        }
    }
    let mut lock = awaiting_ack_clone.lock().await;
    *lock = false;
    println!("WARNING: NO ACK received - Last sent message may not have been received by SC.");
}

/// Function to save downlinked data to a file
fn save_data_to_file(data: Vec<u8>, src: u8) -> std::io::Result<()> {
    let mut dir_name = match ComponentIds::try_from(src) {
        Ok(c) => format!("{c}"),
        Err(_) => "misc".to_string(),
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
fn process_bulk_messages(bulk_messages: Vec<Msg>, num_bytes: usize) -> Result<Msg, &'static str> {
    let mut reconstructed_large_msg = reconstruct_msg(bulk_messages)?;
    reconstructed_large_msg.msg_body = reconstructed_large_msg.msg_body[0..num_bytes].to_vec();
    Ok(reconstructed_large_msg)
}

fn beacon_listen(esat_beacon_interface: &mut TcpInterface) {
    // This function takes a tcp client connected to the simulated uhf's beacon server
    // it reads the buffer and if it is not empty the contents are deserialized into a message
    // right now it is just printing the contents of message to stdout.
    let mut buff = [0; 128];
    let bytes_read = match esat_beacon_interface.read(&mut buff) {
        // If we read no bytes just return early
        Ok(0) => return,
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error reading bytes from UHF beacon port");
            eprintln!("{}", e);
            return;
        }
    };
    if bytes_read > 0 {
        let beacon_msg = from_utf8(&buff).unwrap();
        // Hardcoded slice for now, this code will change in future
        let call_sign_len = 7;
        let call_sign = &beacon_msg[..call_sign_len];
        let content = &beacon_msg[call_sign_len..];
        println!("Beacon ({} bytes): {} {}", bytes_read, call_sign, content);
    }
}

#[tokio::main]
async fn main() {
    let ipaddr = std::env::args().nth(1).unwrap_or("localhost".to_string());

    eprintln!("Connecting to UHF channel via TCP at {ipaddr}...");
    // Create tcp client listening to simulated uhf server.
    let mut esat_uhf_interface =
        match TcpInterface::new_client(ipaddr.to_string(), ports::SIM_ESAT_UHF_PORT) {
            Ok(ti) => ti,
            Err(e) => {
                eprintln!("Can't connect to satellite: {e}");
                process::exit(1);
            }
        };

    eprintln!("Connecting to beacon broadcast channel via TCP at {ipaddr}...");
    // Create tcp client listening to simulated uhf beacon server.
    let mut esat_beacon_interface =
        match TcpInterface::new_client(ipaddr.to_string(), ports::SIM_ESAT_BEACON_PORT) {
            Ok(ti) => ti,
            Err(e) => {
                eprintln!("Can't connect to beacon port: {e}");
                process::exit(1);
            }
        };

    let mut bulk_messages: Vec<Msg> = Vec::new();
    let stdin_fd = std::io::stdin().as_raw_fd();

    loop {
        let mut fds = [libc::pollfd {
            fd: stdin_fd,
            events: POLLIN,
            revents: 0,
        }];

        // Poll stdin for input
        let ret = unsafe { poll(fds.as_mut_ptr(), 1, STDIN_POLL_TIMEOUT) }; // 10 ms timeout
        if ret > 0 && fds[0].revents & POLLIN != 0 {
            let mut input = String::new();
            let mut stdin = std::io::stdin().lock();
            stdin.read_line(&mut input).unwrap();
            let input = input.trim().to_string();

            if let Some(msg) = build_msg_from_operator_input(input) {
                send_msg_to_sc(msg, &mut esat_uhf_interface);

                let mut buf = [0u8; 128];
                let awaiting_ack = Arc::new(Mutex::new(true));
                let awaiting_ack_clone = Arc::clone(&awaiting_ack);

                tokio::task::spawn(async move {
                    awaiting_ack_timeout_task(awaiting_ack_clone).await;
                });

                println!("waiting for ack");
                match esat_uhf_interface.read(&mut buf) {
                    Ok(len) => {
                        if len == 0 {
                            println!("satellite connection ended");
                        }
                        else {
                            match deserialize_msg(&buf) {
                                Ok(response) => handle_response(&response),
                                Err(e) => println!("Response garbled: {}", e),
                            };
                        }
                    },
                    Err(e) => println!("read from satellite failed: {}", e),
                }
                *awaiting_ack.lock().await = false;
            }
        } else {
            // Listens on beacon channel for any beacons we get and prints beacon msg to stdout
            beacon_listen(&mut esat_beacon_interface);

            let mut read_buf = [0; 128];

            let bytes_received = match esat_uhf_interface.read(&mut read_buf) {
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
                    let bytes = [
                        recvd_msg.msg_body[2],
                        recvd_msg.msg_body[3],
                        recvd_msg.msg_body[4],
                        recvd_msg.msg_body[5],
                        recvd_msg.msg_body[6],
                        recvd_msg.msg_body[7],
                        recvd_msg.msg_body[8],
                        recvd_msg.msg_body[9],
                    ];
                    let num_bytes_to_recv = u64::from_le_bytes(bytes);
                    // build_and_send_ack(
                    //     &mut tcp_interface,
                    //     recvd_msg.header.msg_id.clone(),
                    //     recvd_msg.header.source_id,
                    //     recvd_msg.header.dest_id.clone(),
                    // );
                    // Listening mode for bulk msgs
                    bulk::read_msgs(
                        &mut esat_uhf_interface,
                        &mut bulk_messages,
                        num_msgs_to_recv,
                    )
                    .unwrap();

                    // clone bulk_messages BUT maybe hurts performance if there's tons of packets
                    match process_bulk_messages(bulk_messages.clone(), num_bytes_to_recv as usize) {
                        Ok(large_msg) => {
                            println!("Successfully reconstructed 4KB messages");
                            match save_data_to_file(large_msg.msg_body, large_msg.header.source_id)
                            {
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
                let recvd_msg_chars = match String::from_utf8(recvd_msg.msg_body.clone()) {
                    Ok(chars) => Ok(chars),
                    Err(e) => {
                        eprintln!("Couldn't convert recieved message body to UTF8 string: {e}");
                        Err("")
                    }
                };
                println!(
                    "Received Message: {:?}, body {:?} = {:?}",
                    recvd_msg.header, recvd_msg.msg_body, recvd_msg_chars
                );
            } else {
                // Deallocate memory of these messages. Reconstructed version
                // has been written to a file. This is slightly slower than .clear() though
                bulk_messages = Vec::new();
            }
        }
    }
}
