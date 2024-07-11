use common::ports::BULK_MSG_HANDLER_DISPATCHER_PORT;
/*  Writte by Rowan Rasmusson
    Summer 2024
    This program is meant to take serialized Msg Struct and determine
    whether its msg_body is larger than one packet size (128 bytes).
    It will break it into multiple packets if this condition is true and
    will assign the packets a sequence number at msg_body[0]
 */
use tcp_interface::*;
use message_structure::*;

use std::sync::mpsc;
const MAX_BODY_SIZE: usize = 123;
fn main() {
    let large_msg: Msg = Msg::new(2, 5, 1, 5, vec![0; 500]);
    handle_large_msg(large_msg);
}

fn handle_large_msg(large_msg: Msg) -> Vec<Msg> {
    // let ip = "127.0.0.1".to_string();
    // let port = BULK_MSG_HANDLER_DISPATCHER_PORT;
    // let tcp_interface = interfaces::TcpInterface::new_server(ip, port).unwrap();

    // let (bulk_reader_tx, bulk_reader_rx) = mpsc::channel();
    // // let (bulk_writer_tx, bulk_writer_rx) = mpsc::channel();

    // interfaces::async_read(tcp_interface.clone(), bulk_reader_tx, 2048);

    let body_len: usize = large_msg.msg_body.len();

    let mut messages: Vec<Msg> = Vec::new();

    if body_len <= MAX_BODY_SIZE {
        messages.push(large_msg);
    } else {
        let number_of_packets: usize = (body_len + MAX_BODY_SIZE - 1) / MAX_BODY_SIZE;
        let number_of_packets_u8: u8 = number_of_packets as u8;

        // First message with the number of packets
        let first_msg = deconstruct_msg(large_msg.clone(), 0, Some(number_of_packets_u8));
        messages.push(first_msg.clone());
        assert_eq!(first_msg.msg_body[0], number_of_packets_u8);
        // Subsequent messages with chunks of the body
        for i in 0..number_of_packets {
            let start: usize = i * MAX_BODY_SIZE;
            let end: usize = ((i + 1) * MAX_BODY_SIZE).min(body_len);
            let mut msg_part: Msg = large_msg.clone();
            msg_part.msg_body = msg_part.msg_body[start..end].to_vec();
            let chunk_msg: Msg = deconstruct_msg(msg_part, (i + 1) as u8, None);
            messages.push(chunk_msg);
        }
    }
    messages

    // Now we have vector of messages...

}

// return a Msg structure that has
fn deconstruct_msg(mut msg: Msg, sequence_num: u8, total_packets: Option<u8>) -> Msg {
    let head = msg.header;

    if let Some(total) = total_packets {
        msg.msg_body = vec![total];
    } else {
        msg.msg_body.insert(0, sequence_num);
    }

    let body: &[u8] = &msg.msg_body[0..MAX_BODY_SIZE.min(msg.msg_body.len())];
    let sized_msg = Msg {
        header: head,
        msg_body: body.to_vec(),
    };

    println!("Sequence #{}", sequence_num);
    println!("{:?}", sized_msg);

    sized_msg
}

// This is receive large messages from the UHF and be able to put it together to read as one message
fn reconstruct_msg(messages: Vec<Msg>) -> Result<Msg, &'static str> {
    if messages.is_empty() {
        return Err("No messages to reconstruct");
    }

    let first_msg = &messages[0];
    if first_msg.msg_body.is_empty() {
        return Err("First message body empty");
    }

    let total_packets = first_msg.msg_body[0] as usize;
    if total_packets != messages.len() - 1 {
        return Err("Mismatch between number of packets and message count");
    }
    let mut full_body: Vec<u8> = Vec::new();

    for (i,msg) in messages.iter().skip(1).enumerate() {
        if msg.msg_body.is_empty() || msg.msg_body[0] as usize != i + 1 {
            return Err("Invalid sequence number or empty message body");
        }
        full_body.extend_from_slice(&msg.msg_body[1..]);
    }
    Ok(Msg {
        header: first_msg.header.clone(),
        msg_body: full_body,
    })
}

#[cfg(test)]
mod tests {

    use core::num;

    use super::*;

    #[test]
    fn large_msg_copying() {
        let large_msg: Msg = Msg::new(2,5,1,5,vec![0; 500]);
        let messages: Vec<Msg> = handle_large_msg(large_msg.clone());
        assert_eq!(messages[1].msg_body[0], 1);
        assert_eq!(messages[2].msg_body[0], 2);
        assert!(messages[0].header.dest_id == messages[1].header.dest_id);
    }

    #[test]
    fn test_msg_vector_len() {
        let large_msg: Msg = Msg::new(2,5,1,5,vec![0; 742]);
        let messages: Vec<Msg> = handle_large_msg(large_msg.clone());
        let number_of_packets: usize = (large_msg.msg_body.len() + MAX_BODY_SIZE - 1) / MAX_BODY_SIZE;
        assert_eq!(messages.len(), number_of_packets + 1);
    }
}
