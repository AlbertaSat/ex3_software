use common::message_structure::{deserialize_msg, serialize_msg, Msg};
use serialport::{ClearBuffer, SerialPort, SerialPortBuilder};
use std::io::{Error as IoError, Read, Write};

// Interface to send and read Msg structures
pub trait Interface {
    fn read_msg(&mut self) -> Result<Msg, IoError>;
    fn write_msg(&mut self, msg: &Msg) -> Result<usize, IoError>;
}

// Uart interface struct.
pub struct UARTInterface {
    interface: Box<dyn SerialPort>,
}

impl Interface for UARTInterface {
    // reads a Msg structure from interface, returns a Msg struct.
    fn read_msg(&mut self) -> Result<Msg, IoError> {
        let mut bytes = Vec::new();
        self.read(&mut bytes)?;
        deserialize_msg(&bytes)
    }

    // sends a Msg structure to interface, returns the bytes written.
    fn write_msg(&mut self, msg: &Msg) -> Result<usize, IoError> {
        let bytes = serialize_msg(msg)?;
        self.write(&bytes)
    }
}

impl UARTInterface {
    // Constructor for UARTInterface, Opens port with default settings.
    // Default settings are as such:
    // path: MUST BE USER SPECIFIED
    // baud rate: MUST BE USER SPECIFIED
    // data bits: DataBits::Eight
    // flow control: FlowControl::None
    // parity: Parity::None
    // stop bits: StopBits::One
    // timeout: Duration::from_millis(0)
    pub fn new(device_name: &str, baud_rate: u32) -> Result<UARTInterface, IoError> {
        let port_with_settings = serialport::new(device_name, baud_rate);
        let interface = port_with_settings.open()?;
        let uart_interface = UARTInterface { interface };
        Ok(uart_interface)
    }

    // opens a port with the settings specified by the user.
    // to find how to construct a SerialPortBuilder see https://docs.rs/serialport/latest/serialport/
    // This is where users can modify data bits, stop bits, parity, flow control, and timeout
    // parameters.
    pub fn new_with_settings(
        port_with_settings: SerialPortBuilder,
    ) -> Result<UARTInterface, IoError> {
        let interface = port_with_settings.open()?;
        Ok(UARTInterface { interface })
    }

    // reads raw bytes from uart interface into buffer, returns the number of bytes read.
    // Note that if the number of available bytes in the buffer is less than the size of the
    // buffer, then the remaining portion of the buffer is unmodified.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, IoError> {
        let result = self.interface.read(buffer);
        self.clear_input_buffer()?;
        result
    }

    // sends raw bytes in buffer to uart interface, returns the number of bytes written.
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, IoError> {
        self.interface.write(buffer)
    }

    // getter for device name
    pub fn get_device_name(&self) -> String {
        self.interface.name().unwrap()
    }

    // getter for baud_rate
    pub fn get_baud_rate(&self) -> u32 {
        self.interface.baud_rate().unwrap()
    }

    // Checks the outgoing buffer for the amount of bytes to write, returns the number of bytes in
    // the output buffer.
    pub fn available_to_write(&mut self) -> Result<u32, IoError> {
        let bytes_to_write = self.interface.bytes_to_write()?;
        Ok(bytes_to_write)
    }

    // Checks the input buffer for the amount of bytes to read, returns the number of bytes in the
    // buffer
    pub fn available_to_read(&mut self) -> Result<u32, IoError> {
        let bytes_to_read = self.interface.bytes_to_read()?;
        Ok(bytes_to_read)
    }

    // Clears the input buffer
    pub fn clear_input_buffer(&self) -> Result<(), IoError> {
        self.interface.clear(ClearBuffer::Input)?;
        Ok(())
    }

    // Clears the output buffer
    pub fn clear_output_buffer(&self) -> Result<(), IoError> {
        self.interface.clear(ClearBuffer::Output)?;
        Ok(())
    }
}
