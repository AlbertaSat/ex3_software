/*
Written by Devin Headrick and Rowan Rasmusson:

This source file contains a message struct that defines various data formats as
it flows through the ex3 software stack (GS to OBC and various software components within).

References:
    - https://crates.io/crates/serde_json/1.0.1
    - https://crates.io/crates/serde-pickle
*/
use std::io::Error as IoError;

/// This message header is shared by all message types
#[derive(Debug, Clone)]
pub struct MsgHeader {
    pub msg_len: u8,
    pub msg_id: u8,
    pub dest_id: u8,
    pub source_id: u8,
    pub op_code: u8,
}

impl MsgHeader {
    fn to_bytes(&self) -> Result<Vec<u8>, IoError> {
        let mut bytes = Vec::new();
        bytes.push(self.msg_len);
        bytes.push(self.msg_id);
        bytes.push(self.dest_id);
        bytes.push(self.source_id);
        bytes.push(self.op_code);
        Ok(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, IoError> {
        if bytes.len() < 5 {
            return Err(IoError::new(std::io::ErrorKind::InvalidData, "Header bytes too short"));
        }

        Ok(MsgHeader {
            msg_len: bytes[0],
            msg_id: bytes[1],
            dest_id: bytes[2],
            source_id: bytes[3],
            op_code: bytes[4],
        })
    }
}

/// Message struct with header and body
#[derive(Debug, Clone)]
pub struct Msg {
    pub header: MsgHeader,
    pub msg_body: Vec<u8>,
}

impl Msg {
    pub fn new(msg_id: u8, dest_id: u8, source_id: u8, opcode: u8, data: Vec<u8>) -> Self {
        let len = data.len() as u8;
        let header = MsgHeader {
            msg_len: len + 5, // 5 bytes for header fields
            msg_id,
            dest_id,
            source_id,
            op_code: opcode,
        };
        Msg { header, msg_body: data }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, IoError> {
        let mut bytes = self.header.to_bytes()?;
        bytes.extend_from_slice(&self.msg_body);
        Ok(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, IoError> {
        let header_bytes = &bytes[0..5];
        let msg_body = bytes[5..].to_vec();
        let header = MsgHeader::from_bytes(header_bytes)?;
        Ok(Msg { header, msg_body })
    }
}


/// Serialize Msg struct to bytes
pub fn serialize_msg(msg: &Msg) -> Result<Vec<u8>, IoError> {
    let bytes = msg.to_bytes()?;
    Ok(bytes)
}

/// Deserialize bytes into Msg struct
pub fn deserialize_msg(bytes: &[u8]) -> Result<Msg, IoError> {
    let msg: Msg = Msg::from_bytes(bytes)?;
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let msg = Msg::new(1, 2, 3, 4, vec![0, 1, 2, 3, 4, 5, 6]);

        // Serialize
        let serialized_msg_result = msg.to_bytes();
        assert!(serialized_msg_result.is_ok(), "Serialization failed");
        let serialized_msg = serialized_msg_result.unwrap();

        // Deserialize
        let deserialized_msg_result = Msg::from_bytes(&serialized_msg);
        assert!(deserialized_msg_result.is_ok(), "Deserialization failed");
        let deserialized_msg = deserialized_msg_result.unwrap();

        // Assert equality
        assert_eq!(deserialized_msg.header.msg_len, 12);
        assert_eq!(deserialized_msg.msg_body, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_serialize_empty_body() {
        let msg = Msg::new(1, 2, 3, 4, vec![]);

        // Serialize
        let serialized_msg_result = msg.to_bytes();
        assert!(serialized_msg_result.is_ok(), "Serialization failed");
        let serialized_msg = serialized_msg_result.unwrap();

        // Deserialize
        let deserialized_msg_result = Msg::from_bytes(&serialized_msg);
        assert!(deserialized_msg_result.is_ok(), "Deserialization failed");
        let deserialized_msg = deserialized_msg_result.unwrap();

        // Assert equality
        assert_eq!(deserialized_msg.header.msg_len, 5);
        assert_eq!(deserialized_msg.msg_body, vec![]);
    }

    #[test]
    fn test_serialize_max_length_body() {
        // Create a message with the maximum possible body size
        let max_body_size = u8::MAX as usize - 5; // Maximum u8 value minus header size
        let msg = Msg::new(1, 2, 3, 4, vec![0; max_body_size]);

        // Serialize
        let serialized_msg_result = msg.to_bytes();
        assert!(serialized_msg_result.is_ok(), "Serialization failed");
        let serialized_msg = serialized_msg_result.unwrap();

        // Deserialize
        let deserialized_msg_result = Msg::from_bytes(&serialized_msg);
        assert!(deserialized_msg_result.is_ok(), "Deserialization failed");
        let deserialized_msg = deserialized_msg_result.unwrap();

        // Assert equality
        assert_eq!(deserialized_msg.header.msg_len, u8::MAX);
        assert_eq!(deserialized_msg.msg_body.len(), max_body_size);
        assert_eq!(deserialized_msg.msg_body, vec![0; max_body_size]);
    }

    #[should_panic]
    #[test]
    fn test_deserialize_invalid_data() {
        // Provide insufficient bytes for header
        let bytes = vec![0, 1, 2];
    
        // Deserialize should fail
        let deserialized_msg_result = Msg::from_bytes(&bytes);
        assert!(deserialized_msg_result.is_err(), "Deserialization succeeded unexpectedly");
        let err = deserialized_msg_result.err().unwrap();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

}
