use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

// Interface to be implemented by all external interfaces
pub trait Interface {
    // send data as bytes to the interface as a shared byte type slice.
    // Returns the number of bytes sent
    fn send(&mut self, data: &[u8]) -> Result<(), LinuxI2CError>;

    // Read byte data from the interfaace into a byte slice buffer.
    // Returns number of bytes read
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), LinuxI2CError>;
}

// Structure for I2C Interface, i2c is the actual interface while the slave address is where data
// is going to be written and/or read from
pub struct I2cDeviceInterface {
    device: LinuxI2CDevice,
    bus_path: String,
    client_address: u16,
}

impl Interface for I2cDeviceInterface {
    // generic method for sending data over i2c bus to i2c device
    fn send(&mut self, data: &[u8]) -> Result<(), LinuxI2CError> {
        // hacky  way to return the number of bytes written, this is because the i2cdev crate
        // doesn't return any info on how many bytes are sent
        self.device.write(data)
    }

    // generic method for reading data over i2c bus to i2c device
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), LinuxI2CError> {
        self.device.read(buffer)
        // hacky  way to return the number of bytes written, this is because the i2cdev crate
        // doesn't return any info on how many bytes are sent
    }
}

impl I2cDeviceInterface {
    pub fn new(path: &str, client_address: u16) -> Result<I2cDeviceInterface, LinuxI2CError> {
        let device = LinuxI2CDevice::new(path, client_address)?;
        Ok(I2cDeviceInterface {
            device,
            bus_path: path.to_string(),
            client_address,
        })
    }

    // This function writes a single byte to a specific register of a SMbus device
    fn send_byte(&mut self, register: u8, byte: u8) -> Result<(), LinuxI2CError> {
        self.device.smbus_write_byte_data(register, byte)
    }

    // This function reads a single byte from a specific register of a SMbus device
    fn read_byte(&mut self, address: u8) -> Result<u8, LinuxI2CError> {
        self.device.smbus_read_byte_data(address)
    }
}
