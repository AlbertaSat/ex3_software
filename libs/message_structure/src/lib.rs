/*
Written by Devin Headrick and Rowan Rasmusson:

This source file contains a message struct that defines various data formats as
it flows through the ex3 software stack (GS to OBC and various software components within).

References:
    - https://crates.io/crates/serde_json/1.0.1
*/
use serde::Deserialize;
use serde::Serialize;
use serde_json::Error as SerdeError;
use std::io::{Cursor, Error as IoError};

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
    pub fn new(msg_id: u8, dest_id: u8, source_id: u8, opcode: u8, data: Vec<u8>) -> Self {
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

#[derive(Debug)]
pub enum DeserializeError {
    Io(IoError),
    Serde(SerdeError),
}

#[derive(Debug)]
pub enum SerializeError {
    Io(IoError),
    Serde(SerdeError),
}

impl From<IoError> for DeserializeError {
    fn from(error: IoError) -> Self {
        DeserializeError::Io(error)
    }
}

impl From<SerdeError> for DeserializeError {
    fn from(error: SerdeError) -> Self {
        DeserializeError::Serde(error)
    }
}

impl From<IoError> for SerializeError {
    fn from(error: IoError) -> Self {
        SerializeError::Io(error)
    }
}

impl From<SerdeError> for SerializeError {
    fn from(error: SerdeError) -> Self {
        SerializeError::Serde(error)
    }
}

pub fn serialize_msg(msg: Msg) -> Result<Vec<u8>,SerializeError> {
    let mut buf = Vec::new();
    let _serialized_msg = serde_json::to_writer(&mut buf, &msg)?;
    Ok(buf)
}

pub fn deserialize_msg(buffer: Vec<u8>) -> Result<Msg, DeserializeError> {
    // Trimming trailing 0's so JSON doesn't give a "trailing characters" error
    let trimmed_buffer: Vec<_> = buffer.into_iter().take_while(|&x| x != 0).collect();
    let mut cursor = Cursor::new(trimmed_buffer);

    // Deserialize the message
    let deserialized_msg: Msg = serde_json::from_reader(&mut cursor)?;
    Ok(deserialized_msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::ser::{Serialize, Serializer};
    use serde::de::{self, Deserialize, Deserializer};

    #[test]
    fn test_msg_serdes() {
        let msg = Msg::new(0, 2, 3, 4, vec![0, 1, 2, 3, 4, 5, 6]);

        let serialized_msg = serialize_msg(msg).unwrap();
        println!("Serde Msg: {:?}", serialized_msg);

        let deserialized_msg = deserialize_msg(serialized_msg);

        println!("Deserialized Msg: {:?}", deserialized_msg);
        assert_eq!(deserialized_msg.as_ref().unwrap().header.msg_len, 12);
        assert_eq!(deserialized_msg.unwrap().msg_body, vec![0, 1, 2, 3, 4, 5, 6]);
    }
    #[derive(Debug)]
    struct AlwaysFail;

    impl Serialize for AlwaysFail {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("Intentional serialization failure"))
        }
    }

    impl<'de> Deserialize<'de> for AlwaysFail {
        fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Err(de::Error::custom("Intentional deserialization failure"))
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct MsgWithFail {
        header: MsgHeader,
        msg_body: Vec<u8>,
        fail_field: AlwaysFail, // This field will cause serialization to fail
    }

    #[test]
    fn test_msg_serialize_failure() {
        let header = MsgHeader {
            msg_len: 5,
            msg_id: 0,
            dest_id: 2,
            source_id: 3,
            op_code: 4,
        };
        let msg_with_fail = MsgWithFail {
            header,
            msg_body: vec![0, 1, 2, 3, 4, 5, 6],
            fail_field: AlwaysFail,
        };

        let result = serde_json::to_vec(&msg_with_fail);
        assert!(result.is_err(), "Expected serialization to fail");
    }

    #[test]
    fn test_msg_deserialize_failure() {
        // Invalid JSON data
        let invalid_json = b"invalid json data";

        let result: Result<Msg, _> = serde_json::from_slice(invalid_json);
        assert!(result.is_err(), "Expected deserialization to fail");
    }
}
