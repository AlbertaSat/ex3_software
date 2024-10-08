extern crate i2cdev;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::io::{Error, Read, Write};

// Interface to be implemented by all external interfaces
pub trait Interface {
    // send data as bytes to the interface as a shared byte type slice.
    // Returns the number of bytes sent
    fn send(&mut self, data: &[u8]) -> Result<usize, LinuxI2CError>;

    // Read byte data from the interfaace into a byte slice buffer.
    // Returns number of bytes read
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, LinuxI2CError>;
}

// Structure for I2C Interface, i2c is the actual interface while the slave address is where data
// is going to be written and/or read from
pub struct I2CInterface {
    i2c: LinuxI2CDevice,
}

impl Interface for I2CInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize, LinuxI2CError> {
        // Send data to the slave using the i2c interface
        self.i2c.write(data)?;
        Ok(data.len())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, LinuxI2CError> {
        // read data from the slave using the i2c interface
        self.i2c.read(buffer)?;
        Ok(buffer.len())
    }
}

impl I2CInterface {
    pub fn new(path: &str, slave_address: u16) -> Result<I2CInterface, LinuxI2CError> {
        // Initalize i2c interface with path and then pass slave address
        let i2c = LinuxI2CDevice::new(path, slave_address)?;
        Ok(I2CInterface { i2c })
    }
}
