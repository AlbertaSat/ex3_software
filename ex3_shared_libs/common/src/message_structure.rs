/*
Written by Devin Headrick and Rowan Rasmusson:
Summer 2024

This source file contains a message struct that defines various data formats as
it flows through the ex3 software stack (GS to OBC and various software components within).

References:
    - https://crates.io/crates/serde_json/1.0.1
    - https://crates.io/crates/serde-pickle
*/
use crate::component_ids::ComponentIds;
use std::fmt;
use std::io::Error as IoError;

//TODO - add ref to common component id

/// Used when passing messages between around - between components and between GS and SC
pub trait SerializeAndDeserialize {
    fn serialize_to_bytes(&self) -> Result<Vec<u8>, IoError>;
    fn deserialize_from_bytes(byte_vec: Vec<u8>) -> Result<Self, IoError>
    where
        Self: Sized; //Must be sized so compiler can allocated enough space for instances of this type on the stack
}

#[derive(Debug, Clone, Copy)]
pub enum MsgType {
    Cmd = 0,
    Ack = 1,
    Bulk = 2,
    //.. Scheduled msg?
}

impl fmt::Display for MsgType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MsgType::Cmd => write!(f, "Cmd"),
            MsgType::Ack => write!(f, "Ack"),
            MsgType::Bulk => write!(f, "Bulk"),
        }
    }
}

//Convert byte equivalent value to MsgType enum
impl From<u8> for MsgType {
    fn from(byte_val: u8) -> Self {
        match byte_val {
            0 => MsgType::Cmd,
            1 => MsgType::Ack,
            2 => MsgType::Bulk,
            _ => panic!("Invalid MsgType byte value"),
        }
    }
}

//EVERY message should have this header - they all need an id, a dest, and a source
pub struct MsgHeaderNew {
    pub msg_id: u16, // hold up to ~64 thousand unique ids before rollover
    pub msg_type: MsgType,
    pub dest_id: u8,
    pub source_id: u8,
}
impl MsgHeaderNew {
    pub const DEST_INDEX: usize = 3;

    pub fn new(msg_id: u16, msg_type: MsgType, dest_id: u8, source_id: u8) -> Self {
        MsgHeaderNew {
            msg_id,
            msg_type,
            dest_id,
            source_id,
        }
    }
}

/// Use the ComponentId enum to display the source and destination ids actual name
impl fmt::Display for MsgHeaderNew {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MsgId: {},\n\tMsgType: {},\n\tDestId: {},\n\tSourceId: {}",
            self.msg_id,
            self.msg_type,
            ComponentIds::try_from(self.dest_id).unwrap(),
            ComponentIds::try_from(self.source_id).unwrap(),
        )
    }
}

impl SerializeAndDeserialize for MsgHeaderNew {
    fn serialize_to_bytes(&self) -> Result<Vec<u8>, IoError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.msg_id.to_be_bytes());
        bytes.push(self.msg_type as u8);
        bytes.push(self.dest_id);
        bytes.push(self.source_id);
        Ok(bytes)
    }
    fn deserialize_from_bytes(serialized_bytes_slice: Vec<u8>) -> Result<Self, IoError> {
        let result_msg_header = MsgHeaderNew {
            msg_id: u16::from_be_bytes([serialized_bytes_slice[0], serialized_bytes_slice[1]]),
            msg_type: MsgType::from(serialized_bytes_slice[2]),
            dest_id: serialized_bytes_slice[MsgHeaderNew::DEST_INDEX],
            source_id: serialized_bytes_slice[4],
        };
        Ok(result_msg_header)
    }
}

/// Command a handler or other OBC FSW component to do something
pub struct CmdMsg {
    pub header: MsgHeaderNew,
    pub opcode: u8,
    pub data: Vec<u8>,
}
impl CmdMsg {
    pub fn new(msg_id: u16, dest_id: u8, source_id: u8, opcode: u8, data: Vec<u8>) -> Self {
        let header = MsgHeaderNew::new(msg_id, MsgType::Cmd, dest_id, source_id);
        CmdMsg {
            header,
            opcode,
            data,
        }
    }
}

// TODO - print associated opcode name
impl fmt::Display for CmdMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CmdMsg:\nHeader: {}, \nOpcode: {}, \nData: {:?}",
            self.header, self.opcode, self.data
        )
    }
}

impl SerializeAndDeserialize for CmdMsg {
    fn serialize_to_bytes(&self) -> Result<Vec<u8>, IoError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.header.serialize_to_bytes()?);
        bytes.push(self.opcode);
        bytes.extend_from_slice(&self.data);
        Ok(bytes)
    }
    fn deserialize_from_bytes(byte_vec: Vec<u8>) -> Result<Self, IoError>
    where
        Self: Sized,
    {
        let header = MsgHeaderNew::deserialize_from_bytes(byte_vec[0..5].to_vec())?;
        let opcode = byte_vec[5];
        let data = byte_vec[6..].to_vec();
        Ok(CmdMsg {
            header,
            opcode,
            data,
        })
    }
}

/// Inform a sender of a message that the message was received and processed successfully
/// - This DOES NOT indicate the command was successful, just that the message was received and processed
///
/// The ACK will have the same ID as the CmdMsg it's responding to
pub struct AckMsg {
    header: MsgHeaderNew,
    ack_code: AckCode,     // Success or Failure
    context_data: Vec<u8>, // Context as to what failed / why
}
impl AckMsg {
    pub fn new(
        msg_id: u16,
        dest_id: u8,
        source_id: u8,
        ack_code: AckCode,
        context_data: Vec<u8>,
    ) -> Self {
        let header = MsgHeaderNew::new(msg_id, MsgType::Ack, dest_id, source_id);
        AckMsg {
            header,
            ack_code,
            context_data,
        }
    }
}

impl fmt::Display for AckMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AckMsg: \nHeader: {}, \nAckCode: {}, \nContextData: {:?}",
            self.header, self.ack_code, self.context_data
        )
    }
}

impl SerializeAndDeserialize for AckMsg {
    fn serialize_to_bytes(&self) -> Result<Vec<u8>, IoError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.header.serialize_to_bytes()?);
        bytes.push(self.ack_code as u8);
        bytes.extend_from_slice(&self.context_data);
        Ok(bytes)
    }
    fn deserialize_from_bytes(byte_vec: Vec<u8>) -> Result<Self, IoError>
    where
        Self: Sized,
    {
        let header = MsgHeaderNew::deserialize_from_bytes(byte_vec[0..5].to_vec())?;
        let ack_code = AckCode::from(byte_vec[5]);
        let context_data = byte_vec[6..].to_vec();
        Ok(AckMsg {
            header,
            ack_code,
            context_data,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AckCode {
    Success = 0,
    Failed = 1,
}

impl From<u8> for AckCode {
    fn from(byte_val: u8) -> Self {
        match byte_val {
            0 => AckCode::Success,
            1 => AckCode::Failed,
            _ => panic!("Invalid AckCode byte value"),
        }
    }
}

impl fmt::Display for AckCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AckCode::Success => write!(f, "Success"),
            AckCode::Failed => write!(f, "Failed"),
        }
    }
}

// ---------------------------------------------------------------------
const HEADER_SIZE: usize = 7;
/// This message header is shared by all message types
#[derive(Debug, Clone)]
pub struct MsgHeader {
    pub msg_type: u8,
    pub msg_id: u8,
    pub dest_id: u8,
    pub source_id: u8,
    pub op_code: u8,
    pub msg_len: u16,
}

impl MsgHeader {
    pub const DEST_INDEX: usize = 2;

    fn to_bytes(&self) -> Result<Vec<u8>, IoError> {
        let mut bytes = vec![self.msg_type, self.msg_id, self.dest_id, self.source_id, self.op_code];
        bytes.extend(self.msg_len.to_le_bytes());
        Ok(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, IoError> {
        if bytes.len() < 5 {
            return Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "Header bytes too short",
            ));
        }

        Ok(MsgHeader {
            msg_type: bytes[0],
            msg_id: bytes[1],
            dest_id: bytes[2],
            source_id: bytes[3],
            op_code: bytes[4],
            msg_len: u16::from_le_bytes([bytes[5], bytes[6]]),
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
    pub fn new(msg_type: u8, msg_id: u8, dest_id: u8, source_id: u8, op_code: u8, data: Vec<u8>) -> Self {
        let msg_len: u16 = (HEADER_SIZE + data.len()) as u16;
        let header = MsgHeader {
            msg_type,
            msg_id,
            dest_id,
            source_id,
            op_code,
            msg_len,
        };
        Msg {
            header,
            msg_body: data,
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, IoError> {
        let mut bytes = self.header.to_bytes()?;
        bytes.extend_from_slice(&self.msg_body);
        Ok(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, IoError> {
        let header = MsgHeader::from_bytes(&bytes[0..HEADER_SIZE])?;
        let msg_body = bytes[HEADER_SIZE..header.msg_len as usize].to_vec(); // don't include trailing nulls in body
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
    fn test_new_msg_print() {
        //Create Command Message
        let cmd_msg = CmdMsg::new(0, 5, 1, 0, vec![0, 1, 2, 3, 4, 5, 6]);
        println!("{}", cmd_msg);

        //Create Ack Message
        let ack_msg = AckMsg::new(0, 5, 1, AckCode::Success, vec![0, 1, 2, 3, 4, 5, 6]);
        println!("{}", ack_msg);
    }

    #[test]
    fn test_new_msg_ser_and_des() {
        //Create Cmd Msg
        let cmd_msg = CmdMsg::new(0, 5, 1, 0, vec![0, 1, 2, 3, 4, 5, 6]);
        let serialize_cmd_msg = cmd_msg.serialize_to_bytes().unwrap();
        let deserialized_cmd_msg = CmdMsg::deserialize_from_bytes(serialize_cmd_msg).unwrap();
        assert_eq!(deserialized_cmd_msg.header.msg_id, 0);
        assert_eq!(deserialized_cmd_msg.header.dest_id, 5);
        assert_eq!(deserialized_cmd_msg.header.source_id, 1);
        assert!(matches!(deserialized_cmd_msg.header.msg_type, MsgType::Cmd));
        assert_eq!(deserialized_cmd_msg.opcode, 0);
        assert_eq!(deserialized_cmd_msg.data, vec![0, 1, 2, 3, 4, 5, 6]);

        let ack_msg = AckMsg::new(0, 5, 1, AckCode::Success, vec![0, 1, 2, 3, 4, 5, 6]);
        let serialize_ack_msg = ack_msg.serialize_to_bytes().unwrap();
        let deserialized_ack_msg = AckMsg::deserialize_from_bytes(serialize_ack_msg).unwrap();
        assert_eq!(deserialized_ack_msg.header.msg_id, 0);
        assert_eq!(deserialized_ack_msg.header.dest_id, 5);
        assert_eq!(deserialized_ack_msg.header.source_id, 1);
        assert!(matches!(deserialized_ack_msg.header.msg_type, MsgType::Ack));
        assert_eq!(deserialized_ack_msg.context_data, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_serialize_deserialize() {
        let msg = Msg::new(0,1, 2, 3, 4, vec![0, 1, 2, 3, 4, 5, 6]);

        // Serialize
        let serialized_msg = serialize_msg(&msg).unwrap();

        // Deserialize
        let deserialized_msg = deserialize_msg(&serialized_msg).unwrap();

        // Assert equality
        assert_eq!(deserialized_msg.header.msg_type, 0);
        assert_eq!(deserialized_msg.msg_body, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_serialize_empty_body() {
        let msg = Msg::new(0,1, 2, 3, 4, vec![]);

        // Serialize
        let serialized_msg_result = msg.to_bytes();
        assert!(serialized_msg_result.is_ok(), "Serialization failed");
        let serialized_msg = serialized_msg_result.unwrap();

        // Deserialize
        let deserialized_msg_result = Msg::from_bytes(&serialized_msg);
        assert!(deserialized_msg_result.is_ok(), "Deserialization failed");
        let deserialized_msg = deserialized_msg_result.unwrap();

        // Assert equality
        assert_eq!(deserialized_msg.header.msg_type, 0);
        //assert_eq!(deserialized_msg.msg_body, vec![]);
    }

    #[test]
    fn test_serialize_max_length_body() {
        // Create a message with the maximum possible body size
        let max_body_size = u8::MAX as usize - 5; // Maximum u8 value minus header size
        let msg = Msg::new(0,1, 2, 3, 4, vec![0; max_body_size]);

        // Serialize
        let serialized_msg_result = msg.to_bytes();
        assert!(serialized_msg_result.is_ok(), "Serialization failed");
        let serialized_msg = serialized_msg_result.unwrap();

        // Deserialize
        let deserialized_msg_result = Msg::from_bytes(&serialized_msg);
        assert!(deserialized_msg_result.is_ok(), "Deserialization failed");
        let deserialized_msg = deserialized_msg_result.unwrap();

        // Assert equality
        assert_eq!(deserialized_msg.header.msg_type, 0);
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
        assert!(
            deserialized_msg_result.is_err(),
            "Deserialization succeeded unexpectedly"
        );
        let err = deserialized_msg_result.err().unwrap();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
