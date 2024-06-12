/*
Written by Devin Headrick and Rowan Rasmusson:

This source file contains a message struct that defines various data formats as
it flows through the ex3 software stack (GS to OBC and various software components within).

References:
    -
*/
use serde::Serialize;
use serde::Deserialize;

/// This message header is shared by all message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgHeader {
    pub msg_len: u8,
    pub msg_id: u8, // This is a unique identifier for each message sent by the GS
    pub dest_id: u8,   //The message dispatcher uses this to determine which destination to send this message
    pub source_id: u8, // Identifies what sent the message for logging/debugging
    pub op_code: u8, // This is a unique code related to some specfic meaning for each destination
}
//


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Msg {
    pub header: MsgHeader,
    pub msg_body: Vec<u8>, //Contents of the msg body are dictated by the opcode
}

// Have methods to extract different values from a message
// Getting specific values from the msg body based on byte offset is handled by the respective architecture component
// The positions of the fields of the message header

// Define methods that should be the same for all Message types

    // members fxn to get values

    ///TODO - fxn to compare t
impl Msg {
    //Builder that creates a message based on its 'type'
    fn new(
        msg_id: u8,
        dest_id: u8,
        source_id: u8,
        opcode: u8,
        data: Vec<u8>,
    ) -> Self {
        // Build message header first
        let len = data.len() as u8;

        let header = MsgHeader {
            msg_len: len + 5,
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

pub fn get_msg_body(msg: &Msg) -> Vec<u8> {
    msg.msg_body.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use serde_json;

    #[test]
    fn test_msg_serdes() {
        let Msg = Msg {
            header: MsgHeader { msg_len: 0 , msg_id: 0, dest_id: 0, source_id: 0, op_code: 0 },
            msg_body: vec![0,1,2,3,4,5,6],
        };
        let mut buf = Vec::new();
        let serialized_msg = serde_json::to_writer(&mut buf, &Msg).unwrap();
        println!("Serde Msg: {:?}", buf);

        let mut cursor = Cursor::new(buf);
        let deserialized_msg: Msg = serde_json::from_reader(&mut cursor).unwrap();

        println!("Deserialized Msg: {:?}", deserialized_msg);
        assert_eq!(deserialized_msg.header.msg_len,Msg.header.msg_len);
        assert_eq!(deserialized_msg.msg_body,Msg.msg_body);
    }
}
