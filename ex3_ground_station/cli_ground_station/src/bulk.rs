use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{fs, thread};
use std::time::Duration;

use common::component_ids::ComponentIds;
use common::message_structure::*;
use common::bulk_msg_slicing::*;
use interface::{Interface, tcp::TcpInterface};

pub fn parse_cmd(input: &[&str]) -> Option<Vec<u8>> {
    match input.len() {
        0 => {
            println!("Missing path for bulk transfer");
            None
        },
        1 => if input[0] == "help" {
            println!("Usage: {} <file path>", ComponentIds::BulkMsgDispatcher);
            None
        }
        else {
            // This is for the Bulk Msg Disp to parse and determine the path it needs to use to get the data
            Some(input[0].as_bytes().to_vec())
        },
        _ => {
            println!("bulk transfers require exactly one <path> argument");
            None
        }
    }
}

pub fn handle_response(msg: &Msg) {
    println!("msg: {:?}", msg);
}

/// Function to represent the state of reading bulk msgs continuously.
/// It modifies the bulk_messages in place by taking a mutable reference.
fn read_msgs(
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
                thread::sleep(Duration::from_micros(10));
                num_msgs_recvd += 1;
            }
        }
    }

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

pub fn process_download(uhf_iface: &mut TcpInterface) {
    let mut bulk_messages = Vec::new();
    let mut read_buf = [0; 128];
    let bytes_received = match uhf_iface.read(&mut read_buf) {
        Ok(len) => len,
        Err(e) => {
            println!("read failed: {e}");
            return;
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
            read_msgs(
                uhf_iface,
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
                        Ok(_) => println!("Data saved to file"),
                        Err(e) => eprintln!("Error writing data to file: {}", e),
                    }
                },
                Err(e) => eprintln!("Error reconstructing 4K messages: {}", e),
            }

            println!("We have {} bulk msgs including initial header msg",
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
        println!("Received Message: {:?}, body {:?} = {:?}",
                 recvd_msg.header, recvd_msg.msg_body, recvd_msg_chars
        );
    }
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
