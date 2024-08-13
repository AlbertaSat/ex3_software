/*  Writte by Rowan Rasmusson
    Summer 2024
    This program is meant to take serialized Msg Struct and determine
    whether its msg_body is larger than one packet size (128 bytes).
    It will break it into multiple packets if this condition is true and
    will assign the packets a sequence number at msg_body[0]
 */
use message_structure::*;
/// This function holds all the functionality for slicing a large msg into many smaller ones.
/// The size that the message is sliced into is configurable by the parameter max_body_size.
/// This parameter DOES NOT account for the size of the header (as of now).
pub fn handle_large_msg(large_msg: Msg, max_body_size: usize) -> Result<Vec<Msg>, std::io::Error> {
    let body_len: usize = large_msg.msg_body.len();
    let mut messages: Vec<Msg> = Vec::new();

    // Adjust for the space required by sequence numbers
    let max_body_size_adjusted = max_body_size - 2;

    // Might be where bytes aren't being inserted properly
    if body_len <= max_body_size_adjusted {
        // Account for sequence numbers to be added
        let first_msg = deconstruct_msg(large_msg.clone(), 0, Some(1), body_len + 2);
        messages.push(first_msg.clone());
        let small_msg = deconstruct_msg(large_msg.clone(), 1, None, body_len + 2);
        messages.push(small_msg);
    } else {
        let number_of_packets: usize = body_len.div_ceil(max_body_size_adjusted);
        let number_of_packets_u16: u16 = number_of_packets as u16;

        // First message with the number of packets
        let first_msg = deconstruct_msg(large_msg.clone(), 0, Some(number_of_packets_u16), max_body_size);
        messages.push(first_msg.clone());

        // Subsequent messages with chunks of the body
        for i in 0..number_of_packets {
            let start: usize = i * max_body_size - 2*i;
            let end: usize = ((i + 1) * max_body_size).min(body_len);

            let mut msg_part: Msg = large_msg.clone();
            msg_part.msg_body = msg_part.msg_body[start..end].to_vec();

            let chunk_msg: Msg = deconstruct_msg(msg_part, (i + 1) as u16, None, max_body_size);
            messages.push(chunk_msg.clone());
        }
    }
    Ok(messages)

}

// return a Msg structure that has
fn deconstruct_msg(mut msg: Msg, sequence_num: u16, total_packets: Option<u16>, max_body_size: usize) -> Msg {
    let mut head = msg.header;

    if let Some(total) = total_packets {
        let len_bytes = total.to_le_bytes();
        msg.msg_body = len_bytes.to_vec();
    } else {
        let sequence_bytes = sequence_num.to_le_bytes();
        msg.msg_body.insert(0, sequence_bytes[0]);
        msg.msg_body.insert(1, sequence_bytes[1]);
    }

    // TODO - is this necessary?
    let body: &[u8] = &msg.msg_body[0..max_body_size.min(msg.msg_body.len())];
    head.msg_len = 7 + body.len() as u16;
    let sized_msg = Msg {
        header: head,
        msg_body: body.to_vec(),
    };
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

    let total_packets = u16::from_le_bytes([messages[0].msg_body[0], messages[0].msg_body[1]]) as usize;
    if total_packets != messages.len() - 1 {
        eprintln!("total {total_packets}, msgs len {}",messages.len());
        return Err("Mismatch between number of packets and message count");
    }
    let mut full_body: Vec<u8> = Vec::new();

    for (i,msg) in messages.iter().skip(1).enumerate() {
        if msg.msg_body.is_empty() {
            return Err("Empty message body");
        } else if u16::from_le_bytes([msg.msg_body[0], msg.msg_body[1]]) as usize != i + 1 {
            eprintln!("Invalid sequence number {}.\nExpected sequence number of {}", u16::from_le_bytes([msg.msg_body[0], msg.msg_body[1]]), i+1);
            return Err("Invalid sequence number");
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
        let mut original_body = Vec::new();
        for i in 0..=408 {
            original_body.push((i % 256) as u8);
        }
        let len = 7 + original_body.len() as u16;
        let large_msg: Msg = Msg::new(0,2,5,1,5,len, original_body);

        // Handle edge case of max body size and length of msg being one off
        let messages: Vec<Msg> = handle_large_msg(large_msg.clone(), 408).unwrap();

        // Sequence numbers
        assert_eq!(messages[1].msg_body[0], 1);
        assert_eq!(messages[2].msg_body[0], 2);
        // same dest id
        assert!(messages[0].header.dest_id == messages[1].header.dest_id);
        println!("{:?}",messages[2]);
        // Make sure we don't lose last 3 data points not included in first 406 byte msg
        assert_eq!(messages[2].msg_body.len(), 5);
    }

    #[test]
    fn test_msg_vector_len() {
        let max_body_size: usize = 40;
        let mut original_body = Vec::new();
        for i in 0..=408 {
            original_body.push((i % 256) as u8);
        }
        let len = 7 + original_body.len() as u16;
        let large_msg: Msg = Msg::new(0,2,5,1,5,len, original_body);
        let messages: Vec<Msg> = handle_large_msg(large_msg.clone(), max_body_size).unwrap();
        let number_of_packets: usize = (large_msg.msg_body.len() + max_body_size - 1) / max_body_size;
        assert_eq!(messages.len(), number_of_packets + 1);
    }

    #[test]
    fn test_small_msg() {
        let max_body_size = 128;
        let small_msg = Msg::new(2,0,7,3,0,7,vec![2,5]);
        let sliced_small = handle_large_msg(small_msg.clone(), max_body_size).unwrap();
        println!("Small vec: {:?}", sliced_small);
        // 1 message in vec
        // TODO - create enum for easing readability of bytes for bulk msgs
        assert_eq!(sliced_small[0].msg_body[0],1);
        // Check initial packet length
        assert_eq!(sliced_small[0].msg_body.len(), 2);
        // Sequence number 1 of data packet
        assert_eq!(u16::from_le_bytes([sliced_small[1].msg_body[0],sliced_small[1].msg_body[1]]), 1);
        // check data within packet
        assert_eq!(sliced_small[1].msg_body[2], 2);
    }

    #[test]
    fn test_reconstruct_large_msg() {
        let mut original_body: Vec<u8> = Vec::new();
        for i in 0..512 { // 0.5KB of data
            original_body.push((i % 256) as u8); // Different numbered bytes
        }
        let len = 7 + original_body.len() as u16;
        let large_msg = Msg::new(2, 2, 5, 1, 5, len, original_body.clone());

        // Handle the large message, slicing it into smaller packets
        let max_body_size = 128; // 128B packets
        let sliced_msgs = handle_large_msg(large_msg.clone(), max_body_size).unwrap();

        // Reconstruct the message from the sliced packets
        let reconstructed_msg = reconstruct_msg(sliced_msgs).expect("Reconstruction failed");

        // Ensure the reconstructed message matches the original message
        assert_eq!(reconstructed_msg.msg_body, original_body, "The reconstructed message does not match the original message");
    }

    // This test represents how data will be sliced and reconstructed when being downlinked from the satellite
    #[test]
    fn test_spacecraft_reconstruct() {
        // Create a large message with unique byte values to check for offsets
        let mut original_body: Vec<u8> = Vec::new();
        for i in 0..6144 { // 6KB of data
            original_body.push((i % 256) as u8);
        }
        let len = 7 + original_body.len() as u16;
        let large_msg = Msg::new(MsgType::Bulk as u8, 2, 7, 3, 0, len, original_body.clone());

        // First, slice the large message into 2KB packets
        let first_level_packets = handle_large_msg(large_msg.clone(), 2048).unwrap(); // 2KB packets

        // Now, further slice each of the 2KB packets into 128B packets
        let mut second_level_packets = Vec::new();
        for packet in first_level_packets {
            let small_packets = handle_large_msg(packet, 128).unwrap(); // 128B packets
            second_level_packets.extend(small_packets);
        }

        // Reconstruct the original 2KB packets from the 128B packets
        let mut reconstructed_2kb_packets = Vec::new();
        
        // The first message in each sliced packet series indicates how many packets follow it.
        // Group them accordingly to reconstruct them in stages.
        let mut i = 0;
        while i < second_level_packets.len() {
            let first_msg = &second_level_packets[i];
            let total_packets = u16::from_le_bytes([first_msg.msg_body[0], first_msg.msg_body[1]]) as usize;
            let next_group = &second_level_packets[i..i + total_packets + 1];
            let reconstructed_msg = reconstruct_msg(next_group.to_vec()).expect("Reconstruction failed");
            reconstructed_2kb_packets.push(reconstructed_msg);
            i += total_packets + 1;
        }

        // Finally, reconstruct the original 6KB message from the 2KB packets
        let reconstructed_large_msg = reconstruct_msg(reconstructed_2kb_packets).expect("Final reconstruction failed");

        // Ensure the final reconstructed message matches the original large message
        assert_eq!(reconstructed_large_msg.msg_body, original_body, "The final reconstructed message does not match the original message");
    }
}
