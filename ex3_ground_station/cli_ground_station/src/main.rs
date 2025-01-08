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

use common::{ports, ComponentIds};
use common::message_structure::*;
use interface::{tcp::*, Interface};

use std::str::from_utf8;

use std::io::Write;
use std::process;
use std::os::fd::{AsFd};
use std::time::Duration;

use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
use strum::IntoEnumIterator;

use std::fs::OpenOptions;
use serde_json::json;
use std::io::{self, BufRead, BufReader};

const STDIN_PFD: usize = 0;
const UHF_PFD: usize = 1;
const BEACON_PFD: usize = 2;

const ACK_TIMEOUT: u64 = 10; // seconds a receiver (GS or SC) will wait before timing out and asking for a resend

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
            // println!("Built msg: {:?}", msg);
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
        return;
    }

    if let Ok(payload) = ComponentIds::try_from(msg.header.source_id) {
        match payload {
            ComponentIds::BulkMsgDispatcher => bulk::handle_response(msg),
            ComponentIds::EPS => eps::handle_response(msg),
            ComponentIds::SHELL => shell::handle_response(msg),
            _ => {
                println!("response from {:?}: {:?}", payload, msg);
            },
        }
    };
}

fn send_cmd(uhf_iface: &mut TcpInterface) {
    let mut input = String::new();
    let stdin = std::io::stdin();
    match stdin.read_line(&mut input) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("stdin read error: {}", e);
            return;
        }
    }

    let input = input.trim().to_string();
    match build_msg_from_operator_input(input) {
        Some(mstruct) => {
            let msg = serialize_msg(&mstruct).unwrap();
            match uhf_iface.send(&msg) {
                Ok(len) => println!("Sent {} bytes to Coms handler", len),
                Err(e) => {
                    eprintln!("Send to Satellite failed: {}", e);
                    return;
                }
            }
        },
        None => return, // No message to send
    };

    let mut buf = [0u8; 128];

    let _ = uhf_iface.stream.set_read_timeout(Some(Duration::from_secs(ACK_TIMEOUT)));

    match uhf_iface.read(&mut buf) {
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
}

fn beacon_listen(beacon_iface: &mut TcpInterface) {
    // This function takes a tcp client connected to the simulated uhf's beacon server
    // it reads the buffer and if it is not empty the contents are deserialized into a message
    // right now it is just printing the contents of message to stdout.
    let mut buff = [0; 128];
    let bytes_read = match beacon_iface.read(&mut buff) {
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

fn main() {
    let ipaddr = std::env::args().nth(1).unwrap_or("localhost".to_string());

    eprintln!("Connecting to UHF channel via TCP at {ipaddr}...");
    // Create tcp client listening to simulated uhf server.
    let mut uhf_iface =
        match TcpInterface::new_client(ipaddr.to_string(), ports::SIM_ESAT_UHF_PORT) {
            Ok(ti) => ti,
            Err(e) => {
                eprintln!("Can't connect to satellite: {e}");
                process::exit(1);
            }
        };

    eprintln!("Connecting to beacon broadcast channel via TCP at {ipaddr}...");
    // Create tcp client listening to simulated uhf beacon server.
    let mut beacon_iface =
        match TcpInterface::new_client(ipaddr.to_string(), ports::SIM_ESAT_BEACON_PORT) {
            Ok(ti) => ti,
            Err(e) => {
                eprintln!("Can't connect to beacon port: {e}");
                process::exit(2);
            }
        };

    let stdin_stream = std::io::stdin();
    let stdin_pfd = PollFd::new(stdin_stream.as_fd(), PollFlags::POLLIN);
    let uhf_stream = uhf_iface.stream.try_clone().unwrap();
    let uhf_pfd = PollFd::new(uhf_stream.as_fd(), PollFlags::POLLIN);
    let beacon_stream = beacon_iface.stream.try_clone().unwrap();
    let beacon_pfd = PollFd::new(beacon_stream.as_fd(), PollFlags::POLLIN);
    let mut fds = [stdin_pfd, beacon_pfd, uhf_pfd];

    loop {
        print!("ex3> ");
        let _ = std::io::stdout().flush();

        fds[STDIN_PFD] = stdin_pfd;
        fds[BEACON_PFD] = beacon_pfd;
        fds[UHF_PFD] = uhf_pfd;

        // Poll stdin for input
        match poll(&mut fds, PollTimeout::NONE) {
            Ok(n) => {
                if n == 0 {
                    eprintln!("Can't timeout with infinite timeout!");
                    continue;
                }
            },
            Err(e) => {
                eprintln!("poll failed! {}", e);
                process::exit(3);
            }
        };

        let stdin_events = fds[STDIN_PFD].revents().expect("Unexpected STDIN event");
        for flag in stdin_events {
            match flag {
                PollFlags::POLLIN => {
                    send_cmd(&mut uhf_iface);
                },
                PollFlags::POLLHUP => {
                    eprintln!("Lost stdin connection");
                    //stdin_pfd = fake_pollfd();
                    todo!();
                }
                pf => {
                    eprintln!("Unexpected stdin flag: {:?}", pf);
                    process::exit(4);
                },
            };
        }

        let beacon_events = fds[BEACON_PFD].revents().expect("Unexpected beacon event");
        for flag in beacon_events {
            match flag {
                PollFlags::POLLIN => {
                    // Listens on beacon channel for any beacons we get and
                    // prints beacon msg to stdout
                    beacon_listen(&mut beacon_iface);
                },
                PollFlags::POLLHUP => {
                    eprintln!("Lost beacon connection");
                    beacon_iface.close();
                    // beacon_pfd = fake_pollfd();
                    todo!();
                },
                pf => {
                    eprintln!("Unexpected beacon flag: {:?}", pf);
                    process::exit(5);
                },
            };
        }

        let uhf_events = fds[UHF_PFD].revents().expect("Unexpected UHF event");
        for flag in uhf_events {
            match flag {
                PollFlags::POLLIN => bulk::process_download(&mut uhf_iface),
                PollFlags::POLLHUP => {
                    eprintln!("Lost UHF connection");
                    beacon_iface.close();
                    // uhf_pfd = fake_pollfd();
                    todo!();
                },
                pf => {
                    eprintln!("Unexpected UHF flag: {:?}", pf);
                    process::exit(6);
                },
            };
        }
    }
}

// @Parameters:
// pipe_path - path of the fifo pipe, should just be "../server/cli_to_server"
// json_struct -  json!({"key1": "value1", "key2":"value2"})
//
// @Example
// write_to_pipe(json!({"key1": "value1", "key2":"value2"}), "../server/cli_to_server")
//
fn send_to_server(json_struct: serde_json::Value, pipe_path: &str) -> std::io::Result<()>{
    let mut pipe = OpenOptions::new().write(true).open(pipe_path)?;

    let data = json!(json_struct);
    let serialized = serde_json::to_string(&data).unwrap();

    pipe.write_all(serialized.as_bytes())?;

    Ok(())
}

// @Parameters
// pipe_path - "server_to_cli"
// 
// @Example
// let value = read_from_pipe("server_to_cli")
// let myStruct: MyStruct = serde_json::from_str(&value).unwrap();
// println!("{}", myStruct.key1)
//
fn recieve_from_server(pipe_path: &str) -> io::Result<String> {
    let pipe = File::open(pipe_path)?;
    let mut reader = BufReader::new(pipe);

    let mut serialized = String::new();
    reader.read_line(&mut serialized)?;

    Ok(serialized)
}