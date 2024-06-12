/*
Written by Devin Headrick and Rowan Rasmusson:

This source file contains a message struct that defines various data formats as
it flows through the ex3 software stack (GS to OBC and various software components within).

References:
    -
*/
use serde::Deserialize;
use serde::Serialize;

/// This message header is shared by all message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgHeader {
    pub msg_len: u8,   // This is the length of the whole message (header + body) in bytes
    pub msg_id: u8,    // This is a unique identifier for each message sent by the GS
    pub dest_id: u8, //The message dispatcher uses this to determine which destination to send this message
    pub source_id: u8, // Identifies what sent the message for logging/debugging
    pub op_code: u8, // This is a unique code related to some specfic meaning for each destination
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Msg {
    pub header: MsgHeader,
    pub msg_body: Vec<u8>, //Contents of the msg body are dictated by the opcode
}

impl Msg {
    // Constructor to create message header with correct length
    fn new(msg_id: u8, dest_id: u8, source_id: u8, opcode: u8, data: Vec<u8>) -> Self {
        let len = data.len() as u8;
        let header = MsgHeader {
            msg_len: len + 5, //5 bytes for header fields
            msg_id: msg_id,
            dest_id: dest_id,
            source_id: source_id,
            op_code: opcode,
        };
        Msg {
            header,
            msg_body: data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::io::Cursor;

    #[test]
    fn test_msg_serdes() {
        let msg = Msg::new(0, 2, 3, 4, vec![0, 1, 2, 3, 4, 5, 6]);
        let mut buf = Vec::new();
        let serialized_msg = serde_json::to_writer(&mut buf, &msg).unwrap();
        println!("Serde Msg: {:?}", buf);

        let mut cursor = Cursor::new(buf);
        let deserialized_msg: Msg = serde_json::from_reader(&mut cursor).unwrap();

        println!("Deserialized Msg: {:?}", deserialized_msg);
        assert_eq!(deserialized_msg.header.msg_len, msg.header.msg_len);
        assert_eq!(deserialized_msg.msg_body, msg.msg_body);
    }
}
