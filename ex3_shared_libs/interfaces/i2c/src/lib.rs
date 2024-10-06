use linux_embedded_hal::{i2c::Error as I2cError, I2cdev};
use std::io::{self, read, write};

// Interface to be implemented by all external interfaces
pub trait Interface {
    // send data as bytes to the interface as a shared byte type slice.
    // Returns the number of bytes sent
    fn send(&mut self, data: &[u8]) -> Result<usize, I2cError>;

    // Read byte data from the interfaace into a byte slice buffer.
    // Returns number of bytes read
    fn read(&mut self, buffer: &mut &[u8]) -> Result<usize, I2cError>;
}

// Structure for I2C Interface, i2c is the actual interface while the slave address is where data
// is going to be written and/or read from
pub struct I2CInterface {
    i2c: I2cdev,
    slave_address: u16,
}

impl Interface for I2CInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize, I2cError> {
        // Send data to the slave using the i2c interface
        self.i2c.write(self.slave_address, data)?;
        Ok(data.len())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, I2cError> {
        // read data from the slave using the i2c interface
        self.i2c.read(self.slave_address, buffer)?;
        Ok(buffer.len())
    }
}

impl I2CInterface {
    pub fn new(path: &str, slave_address: u16) -> io::Result<Self> {
        // Initalize i2c interface with path and then pass slave address
        let i2c = I2cdev::new(path)?;
        Ok(I2CInterface { i2c, slave_address })
    }
}
