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
use core::num;
use std::sync::mpsc;
const MAX_BODY_SIZE: usize = 123;
fn main() {
    run_bulk_msg_handler();
}

fn run_bulk_msg_handler() {
    let ip = "127.0.0.1".to_string();
    let port = BULK_MSG_HANDLER_DISPATCHER_PORT;
    let tcp_interface = interfaces::TcpInterface::new_server(ip, port).unwrap();

    let (bulk_reader_tx, bulk_reader_rx) = mpsc::channel();
    // let (bulk_writer_tx, bulk_writer_rx) = mpsc::channel();

    interfaces::async_read(tcp_interface.clone(), bulk_reader_tx, TCP_BUFFER_SIZE);
    loop {
        let mut body_len: usize = 0;
        if let Ok(buffer) = bulk_reader_rx.recv() {
            let mut deserialized_msg: Msg = deserialize_msg(buffer).unwrap();
            // len() returns the length in bytes here since each element is a u8 (1 byte)
            body_len =  deserialized_msg.msg_body.len();

            if body_len <= MAX_BODY_SIZE {
                // write to stream
            } else {
                let number_of_packets = body_len / MAX_BODY_SIZE;
                for i in 0..number_of_packets {
                    chop_msg(deserialized_msg, i as u8);
                }

            }
        } else {
            eprintln!("Failed to read Msg struct");
        }
        }
    }

// return a Msg structure that has
fn chop_msg(mut msg: Msg, sequence_num: u8) -> Msg {
    let head = msg.header;
    msg.msg_body.insert(0, sequence_num);
    let body: &[u8] = &msg.msg_body[1..MAX_BODY_SIZE];
    let sized_msg = Msg {
        header: head,
        msg_body: body.to_vec(),
    };
    sized_msg

}
