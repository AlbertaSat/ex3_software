use std::thread;
use std::time::Duration;

use common::component_ids::ComponentIds;
use common::message_structure::*;
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

/// Function to represent the state of reading bulk msgs continuously.
/// It modifies the bulk_messages in place by taking a mutable reference.
pub fn read_msgs(
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

                let seq_id = if cur_msg.msg_body.len() >= 2 {
                    // Attempt to read seq_id as u16 if enough bytes are available
                    u16::from_le_bytes([cur_msg.msg_body[0], cur_msg.msg_body[1]])
                } else {
                    // Fallback: use the first byte as seq_id if insufficient bytes
                    cur_msg.msg_body[0] as u16
                };

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


                
