/*
Written by Devin Headrick
Summer 2024


References:
    - CLI book: https://rust-cli.github.io/book/index.html

TODO
    - Test various operator inputs and edge cases
    - Have the 'up' key bring back the previously entered command

*/

use common::component_ids::*;
use common::ports::SIM_COMMS_PORT;
use message_structure::*;
use tcp_interface::*;

use libc::{poll, POLLIN};
use std::io::prelude::*;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use chrono::prelude::*;
use serde_json::json;
use std::thread;
use tokio;
use tokio::io::{self, AsyncBufReadExt, BufWriter, AsyncWriteExt};
use std::io::Write;
use std::str::FromStr;

const WAIT_FOR_ACK_TIMEOUT: u64 = 10; // seconds a receiver (GS or SC) will wait before timing out and asking for a resend

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
    let dest_id = ComponentIds::from_str(operator_str_split[0]).unwrap() as u8;
    let opcode = operator_str_split[1].parse::<u8>().unwrap();
    let mut msg_body: Vec<u8> = Vec::new();
    for data_byte in operator_str_split[2..].into_iter() {
        msg_body.push(data_byte.parse::<u8>().unwrap());
    }

    let msg = Msg::new(0,0, dest_id, GS, opcode, msg_body);
    println!("Built msg: {:?}", msg);
    Ok(msg)
}

/// Blocking io read operator input from stdin, trim, and store command in JSON
async fn get_operator_input_line() -> String {
    let mut input = String::new();
    print!("Ex3 CLI GS > ");
    io::stdout().flush().await.unwrap();
    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    if let Some(line) = lines.next_line().await.unwrap() {
        input = line.trim().to_string();
    }

    // store_operator_entered_string(input.clone());
    input
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
/// It modifies the 
fn read_bulk_msgs(tcp_interface: &mut TcpInterface, mut bulk_messages: Vec<Msg>, num_msgs_to_recv: u16) {
    let mut bytes_read = 0;
    loop{
        println!("Bulking it up");
        let mut bulk_buf = [0u8; 128];
        bytes_read += tcp_interface.read(&mut bulk_buf).unwrap();
        if bytes_read > 0 {
            if (bytes_read as u16) < num_msgs_to_recv*128 {
                let cur_msg = deserialize_msg(&bulk_buf).unwrap();
                println!("Received msg #{}", u16::from_le_bytes([cur_msg.msg_body[0], cur_msg.msg_body[1]]));
                bulk_messages.push(cur_msg.clone());
                thread::sleep(Duration::from_millis(5));
                continue;
            } else {
                todo!()
            }
        } continue;
    }
}

#[tokio::main]
async fn main() {
    println!("Beginning CLI Ground Station...");
    println!("Waiting for connection to Coms handler via TCP...");
    let mut tcp_interface =
        TcpInterface::new_server("127.0.0.1".to_string(), SIM_COMMS_PORT).unwrap();
    println!("Connected to Coms handler via TCP ");

    let mut receiving_bulk = false;
    let mut num_msgs_to_recv: u16 = 0;
    let mut bulk_messages: Vec<Msg> = Vec::new();
    let mut bytes_read = 0;

    let stdin_fd = std::io::stdin().as_raw_fd();

    loop {
        println!("Starting loop");

        // if receiving_bulk {
        //     println!("Bulking it up");
        //     let mut bulk_buf = [0u8; 128];
        //     bytes_read += tcp_interface.read(&mut bulk_buf).unwrap();
        //     if bytes_read > 0 {
        //         if (bytes_read as u16) < num_msgs_to_recv*128 {
        //             let cur_msg = deserialize_msg(&bulk_buf).unwrap();
        //             println!("Received msg #{}", u16::from_le_bytes([cur_msg.msg_body[0], cur_msg.msg_body[1]]));
        //             bulk_messages.push(cur_msg.clone());
        //             sleep(Duration::from_millis(10)).await;
        //             continue;
        //         } else {
        //             todo!()
        //         }
        //     } else {
        //         continue;
        //     }
        // }

        let mut fds = [libc::pollfd {
            fd: stdin_fd,
            events: POLLIN as i16,
            revents: 0,
        }];

        // Poll stdin for input
        let ret = unsafe { poll(fds.as_mut_ptr(), 1, 3000) }; // 3-second timeout
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
            let bytes_received = tcp_interface.read(&mut read_buf).unwrap();
            if bytes_received > 0 {
                let recvd_msg = deserialize_msg(&read_buf).unwrap();
                if recvd_msg.header.msg_type == MsgType::Bulk as u8 && !receiving_bulk {
                    num_msgs_to_recv = u16::from_le_bytes([recvd_msg.msg_body[0], recvd_msg.msg_body[1]]);
                    println!("Num of msgs incoming: {}", num_msgs_to_recv);
                    receiving_bulk = true;
                    println!("Receiving bulk: {}", receiving_bulk);
                    read_bulk_msgs(&mut tcp_interface, bulk_messages.clone(), num_msgs_to_recv);
                }
                println!("Received Data: {:?}", read_buf);
            } else {
                continue;
            }
        }
    }
}
