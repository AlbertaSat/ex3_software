use common::ports::BULK_MSG_HANDLER_DISPATCHER_PORT;
/*  Writte by Rowan Rasmusson
    Summer 2024
    This program is meant to take serialized Msg Struct and determine
    whether its msg_body is larger than one packet size (128 bytes).
    It will break it into multiple packets if this condition is true and
    will assign the packets a sequence number at msg_body[0]
 */
use interfaces::*;
use message_structure::*;
use common::*;
use std::char::MAX;
use std::io::Cursor;
use std::sync::mpsc;
const MAX_BODY_SIZE: u8 = 128;
fn main() {
    run_bulk_msg_handler();
}

fn run_bulk_msg_handler() {
    let ip = "127.0.0.1".to_string();
    let port = BULK_MSG_HANDLER_DISPATCHER_PORT;
    let tcp_interface = interfaces::TcpInterface::new_server(ip, port).unwrap();

    let (sched_reader_tx, sched_reader_rx) = mpsc::channel();
    // let (sched_writer_tx, sched_writer_rx) = mpsc::channel();

    interfaces::async_read(tcp_interface.clone(), sched_reader_tx, TCP_BUFFER_SIZE);

    loop {
        if let Ok(buffer) = sched_reader_rx.recv() {
            // Trimming trailing 0's so JSON doesn't give a "trailing characters" error
            let trimmed_buffer: Vec<_> = buffer.into_iter().take_while(|&x| x != 0).collect();
            let mut cursor = Cursor::new(trimmed_buffer);
            let deserialized_msg: Msg = serde_json::from_reader(&mut cursor).unwrap();

            // gets size of msg
            let body_size: u64 = deserialized_msg.msg_body.len(); // this is # of bytes since each element is a u8

            if body_size <= MAX_BODY_SIZE {
                // let packet go
            } else {
                break_packet();
            }

        }
    }
        }
