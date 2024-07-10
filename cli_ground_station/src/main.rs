/*
Written by Devin Headrick
Summer 2024

The point of this cargo project is to act as a cli version of the Ground Station.

Operators can type the same commands they would enter in the GUI and view the output in the terminal directly.

References:
    - CLI book: https://rust-cli.github.io/book/index.html

TODO
    - Have the 'up' key bring back the previously entered command

*/

use clap::Parser;
use std::io::prelude::*;

use common::component_ids::*;
use common::ports::SIM_COMMS_PORT;
use message_structure::*;
use tcp_interface::*;

use chrono::prelude::*;
use serde_json::json;
use std::io::{BufWriter, Write};

use std::time::Duration;
use tokio;

use std::sync::Arc;
use tokio::sync::Mutex;

const WAIT_FOR_ACK_TIMEOUT: u64 = 10; // seconds a receiver (GS or SC) will wait before timing out and asking for a resend

//TOOD - create a new file for each time the program is run
//TODO - get file if one already this time the 'program is run' - then properly append JSON data (right now it just appends json data entirely)
//TODO - get the current users name
fn store_operator_entered_string(operator_str: String) {
    // Write the operator entered string to a file using JSON, with a time stamp
    let utc: DateTime<Utc> = Utc::now();
    let operator_json = json!({
        "time": utc.to_string(),
        "operator_input": operator_str,
        "user: " : "Operator 1"
    });
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("operator_input.json")
        .unwrap();

    let mut writer = BufWriter::new(&file);
    serde_json::to_writer(&mut writer, &operator_json).unwrap();
    writer.flush();
}

fn build_msg_from_operator_input(operator_str: String) -> Result<Msg, std::io::Error> {
    //Parse input string by spaces
    let operator_str_split: Vec<&str> = operator_str.split(" ").collect();

    if operator_str_split.len() < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Not enough arguments",
        ));
    }

    // 1st value is the destination component string - converted to equivalent component id
    let dest_id = match operator_str_split[0] {
        "OBC" => OBC,
        "EPS" => EPS,
        "ADCS" => ADCS,
        "DFGM" => DFGM,
        "IRIS" => IRIS,
        "GPS" => GPS,
        "COMS" => COMS,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid destination component name",
            ));
        }
    };
    // 2nd value is opcode num - converted from ascii to a byte
    let opcode = operator_str_split[1].parse::<u8>().unwrap();

    // Remaining values are the data - converted from ascii into bytes
    let mut msg_body: Vec<u8> = Vec::new();
    for data_byte in operator_str_split[2..].into_iter() {
        msg_body.push(data_byte.parse::<u8>().unwrap());
    }

    let msg = Msg::new(0, dest_id, GS, opcode, msg_body);
    println!("Built msg: {:?}", msg);
    Ok(msg)
}

/// Blocking io read operator input from stdin, trim, and store command in JSON
fn get_operator_input_line() -> String {
    let mut input = String::new();
    print!("Ex3 CLI GS > ");
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let input = input.trim().to_string();

    // Write the operator entered string to a file using JSON, with a time stamp
    store_operator_entered_string(input.clone());
    return input;
}

// Takes mutable reference to the awaiting ack flag, derefs it and sets the value
fn handle_ack(msg: Msg, awaiting_ack: &mut bool) -> Result<(), std::io::Error> {
    //TODO - hanlde if the Ack is OK or ERR , OR not an ACK at all
    println!("Received ACK: {:?}", msg);
    *awaiting_ack = false;
    Ok(())
}

fn send_msg_to_sc(msg: Msg, tcp_interface: &mut TcpInterface) {
    // Serialize msg to bytes
    let serialized_msg = serialize_msg(&msg).unwrap();

    // Send the message to the coms handler via TCP
    let ret = tcp_interface.send(&serialized_msg).unwrap();
    println!("Sent {} bytes to Coms handler", ret);
    std::io::stdout().flush().unwrap();
}

#[tokio::main]
async fn main() {
    println!("Beginning CLI Ground Station...");
    println!("Waiting for connection to Coms handler via TCP..."); // bypass the UHF transceiver and direct to coms handler for now
    let mut tcp_interface =
        TcpInterface::new_server("127.0.0.1".to_string(), SIM_COMMS_PORT).unwrap();
    println!("Connected to Coms handler via TCP ");

    //Once connection is established, loop and read stdin, build a msg from operator entered data, and send to coms handler via TCP socket
    loop {
        let input = get_operator_input_line(); //Bl;ocking read on stdin until operator hits 'enter' key

        let msg_build_res = build_msg_from_operator_input(input);

        match msg_build_res {
            Ok(msg) => {
                send_msg_to_sc(msg, &mut tcp_interface);

                //TODO - Wait for an ACK w/ a timeout of 10 seconds - if no ACK, inform the operator and ask if they want to resend
                let mut buf = [0u8; 128];
                let awaiting_ack = Arc::new(Mutex::new(true));
                let awaiting_ack_clone = Arc::clone(&awaiting_ack);

                tokio::task::spawn(async move {
                    //Sleep for 1 second intervals - and check if the await ack flag has been reset
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
                });

                while *awaiting_ack.lock().await == true {
                    let mut bytes_read = tcp_interface.read(&mut buf).unwrap();
                    if bytes_read > 0 {
                        let ack_msg = deserialize_msg(&buf).unwrap();
                        let ack_res = handle_ack(ack_msg, &mut *awaiting_ack.lock().await);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error building message: {}", e);
            }
        }
    }
}
