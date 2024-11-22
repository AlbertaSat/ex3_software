use super::Interface;
use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::io::Error as IoError;

// I2c Device structure.
pub struct I2cDeviceInterface {
    device: LinuxI2CDevice,
    bus_path: String,
    client_address: u16,
}

impl Interface for I2cDeviceInterface {
    // sends raw bytes to i2c device, returns the amount of bytes sent
    fn write(&mut self, data: &[u8]) -> Result<usize, IoError> {
        self.device.write(data)?;
        Ok(data.len())
    }

    // reads raw bytes from device into buffer
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, IoError> {
        self.device.read(buffer)?;
        Ok(buffer.len())
    }
}

impl I2cDeviceInterface {
    // Constructor for I2cDeviceInterface struct
    pub fn new(path: &str, client_address: u16) -> Result<I2cDeviceInterface, LinuxI2CError> {
        let device = LinuxI2CDevice::new(path, client_address)?;
        Ok(I2cDeviceInterface {
            device,
            bus_path: path.to_string(),
            client_address,
        })
    }

    // writes a single byte to a specific register of a SMbus device
    pub fn send_byte_smbus(&mut self, register: u8, byte: u8) -> Result<(), LinuxI2CError> {
        self.device.smbus_write_byte_data(register, byte)
    }

    // reads a single byte from a specific register of a SMbus device
    pub fn read_byte_smbus(&mut self, address: u8) -> Result<u8, LinuxI2CError> {
        self.device.smbus_read_byte_data(address)
    }

    // Getter for device's bus path
    pub fn get_bus_path_name(&self) -> String {
        self.bus_path.clone()
    }

    // Getter for clients i2c address number
    pub fn get_client_address(&self) -> u16 {
        self.client_address
    }
}
