use message_structure::{deserialize_msg, serialize_msg, Msg};
use serialport::TTYPort;
use std::io::{Error as IoError, Read, Write};

pub trait Interface {
    fn read(&mut self) -> Result<Msg, IoError>;
    fn send(&mut self, msg: &Msg) -> Result<usize, IoError>;
}

pub struct UARTInterface {
    interface: TTYPort,
    device_name: String,
    baud_rate: u32,
    buffer: Vec<u8>,
}

impl Interface for UARTInterface {
    fn read(&mut self) -> Result<Msg, IoError> {
        let _ = self.interface.read(&mut self.buffer);
        let ret = deserialize_msg(&self.buffer);
        self.buffer.clear();
        ret
    }

    fn send(&mut self, msg: &Msg) -> Result<usize, IoError> {
        self.buffer = serialize_msg(msg)?;
        let ret = self.interface.write(&self.buffer);
        self.buffer.clear();
        ret
    }
}

impl UARTInterface {
    pub fn new(&mut self, device_name: &str, baud_rate: u32) -> Result<UARTInterface, IoError> {
        let interface = TTYPort::open(&serialport::new(device_name, baud_rate))?;
        Ok(UARTInterface {
            interface,
            device_name: device_name.to_string(),
            baud_rate,
            buffer: Vec::new(),
        })
    }

    pub fn read_raw_bytes(&mut self) -> Result<usize, IoError> {
        self.interface.read(&mut self.buffer)
    }

    pub fn write_raw_bytes(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        self.interface.write(buf)
    }

    pub fn get_device_name(&self) -> String {
        self.device_name.clone()
    }

    pub fn get_baud_rate(&self) -> u32 {
        self.baud_rate
    }
}
