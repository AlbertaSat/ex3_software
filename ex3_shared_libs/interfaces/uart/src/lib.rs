use message_structure::{deserialize_msg, serialize_msg, Msg};
use serialport::SerialPort;
use std::io::{Error as IoError, Read, Write};

// Interface to send and read Msg structures
pub trait Interface {
    fn recv_msg(&mut self) -> Result<Msg, IoError>;
    fn write_msg(&mut self, msg: &Msg) -> Result<usize, IoError>;
}

// Uart interface struct.
pub struct UARTInterface {
    interface: Box<dyn SerialPort>,
    device_name: String,
    baud_rate: u32,
}

impl Interface for UARTInterface {
    // reads a Msg structure from interface, returns a Msg struct.
    fn recv_msg(&mut self) -> Result<Msg, IoError> {
        let mut bytes = Vec::new();
        self.recv(&mut bytes)?;
        deserialize_msg(&bytes)
    }

    // sends a Msg structure to interface, returns the bytes written.
    fn write_msg(&mut self, msg: &Msg) -> Result<usize, IoError> {
        let bytes = serialize_msg(msg)?;
        self.write(&bytes)
    }
}

impl UARTInterface {
    // Constructor for UARTInterface.
    pub fn new(device_name: &str, baud_rate: u32) -> Result<UARTInterface, IoError> {
        let interface = serialport::new(device_name, baud_rate).open()?;
        Ok(UARTInterface {
            interface,
            device_name: device_name.to_string(),
            baud_rate,
        })
    }

    // reads raw bytes from uart interface into buffer, returns the number of bytes read.
    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, IoError> {
        self.interface.read(buffer)
    }

    // sends raw bytes in buffer to uart interface, returns the number of bytes written.
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, IoError> {
        self.interface.write(buffer)
    }

    // getter for device name
    pub fn get_device_name(&self) -> String {
        self.device_name.clone()
    }

    // getter for baud_rate
    pub fn get_baud_rate(&self) -> u32 {
        self.baud_rate
    }
}
