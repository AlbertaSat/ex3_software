use core::num;

/*  Writte by Rowan Rasmusson
    Summer 2024
    This program is meant to take serialized Msg Struct and determine
    whether its msg_body is larger than one packet size (128 bytes).
    It will break it into multiple packets if this condition is true and
    will assign the packets a sequence number at msg_body[0]
 */
use message_structure::*;

pub fn handle_large_msg(large_msg: Msg, max_body_size: usize) -> Result<Vec<Msg>, std::io::Error> {

    let body_len: usize = large_msg.msg_body.len();
    assert!(max_body_size < body_len);
    let mut messages: Vec<Msg> = Vec::new();

    if body_len <= max_body_size {
        messages.push(large_msg);
    } else {
        let number_of_packets: usize = body_len.div_ceil(max_body_size);
        let number_of_packets_u16: u16 = number_of_packets as u16;

        // First message with the number of packets
        let first_msg = deconstruct_msg(large_msg.clone(), 0, Some(number_of_packets_u16), max_body_size);
        messages.push(first_msg.clone());
        println!("Messages to be sent: {}", number_of_packets);
        assert_eq!(u16::from_le_bytes([first_msg.msg_body[0], first_msg.msg_body[1]]), number_of_packets_u16);
        // Subsequent messages with chunks of the body
        for i in 0..number_of_packets {
            let start: usize = i * max_body_size;
            let end: usize = ((i + 1) * max_body_size).min(body_len);
            let mut msg_part: Msg = large_msg.clone();
            msg_part.msg_body = msg_part.msg_body[start..end].to_vec();
            let chunk_msg: Msg = deconstruct_msg(msg_part, (i + 1) as u16, None, max_body_size);
            messages.push(chunk_msg);
        }
    }
    Ok(messages)

}

// return a Msg structure that has
fn deconstruct_msg(mut msg: Msg, sequence_num: u16, total_packets: Option<u16>, max_body_size: usize) -> Msg {
    let head = msg.header;

    if let Some(total) = total_packets {
        let len_bytes = total.to_le_bytes();
        msg.msg_body = vec![len_bytes[0], len_bytes[1]];
    } else {
        let sequence_bytes = sequence_num.to_le_bytes();
        msg.msg_body.insert(0, sequence_bytes[0]);
        msg.msg_body.insert(1, sequence_bytes[1]);
    }

    let body: &[u8] = &msg.msg_body[0..max_body_size.min(msg.msg_body.len())];
    let sized_msg = Msg {
        header: head,
        msg_body: body.to_vec(),
    };

    println!("Sequence #{}", sequence_num);
    println!("{:?}", sized_msg);

    sized_msg
}

/// This function receives a vector of large messages from the UHF and be able to put it together to read as one message
pub fn reconstruct_msg(messages: Vec<Msg>) -> Result<Msg, &'static str> {
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
        if msg.msg_body.is_empty() || u16::from_le_bytes([msg.msg_body[0], msg.msg_body[1]]) as usize != i + 1 {
            return Err("Invalid sequence number or empty message body");
        }
        full_body.extend_from_slice(&msg.msg_body[2..]);
    }
    Ok(Msg {
        header: first_msg.header.clone(),
        msg_body: full_body,
    })
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn large_msg_copying() {
        let large_msg: Msg = Msg::new(0,2,5,1,5,vec![0; 409]);
        let messages: Vec<Msg> = handle_large_msg(large_msg.clone(), 408).unwrap();
        assert_eq!(messages[1].msg_body[0], 1);
        assert_eq!(messages[2].msg_body[0], 2);
        assert!(messages[0].header.dest_id == messages[1].header.dest_id);
    }

    #[test]
    fn test_msg_vector_len() {
        let max_body_size: usize = 40;
        let large_msg: Msg = Msg::new(0,2,5,1,5,vec![0; 742]);
        let messages: Vec<Msg> = handle_large_msg(large_msg.clone(), max_body_size).unwrap();
        let number_of_packets: usize = (large_msg.msg_body.len() + max_body_size - 1) / max_body_size;
        assert_eq!(messages.len(), number_of_packets + 1);
    }
}
