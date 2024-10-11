extern crate i2cdev;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::io::{Error, Write};

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
    bus_path: String,
    client_address: u16,
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
    pub fn new(path: &str, client_address: u16) -> Result<I2CInterface, LinuxI2CError> {
        // Initalize i2c interface with path and then pass slave address
        let i2c = LinuxI2CDevice::new(path, client_address)?;
        Ok(I2CInterface {
            i2c,
            bus_path: path.to_string(),
            client_address,
        })
    }

    fn send_sm(&mut self, register: u8, byte: u8) -> Result<(), LinuxI2CError> {
        // Send data to the slave using the i2c interface
        self.i2c.smbus_write_byte_data(register, byte)?;
        Ok(())
    }

    fn read_sm(&mut self, address: u8) -> Result<u8, LinuxI2CError> {
        // read data from the slave using the i2c interface
        self.i2c.smbus_read_byte_data(address)
    }

    fn print_device_registers(&mut self) {
        println!("     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f");

        for row in (0x00..=0xFF).step_by(16) {
            print!("{:02X}: ", row);

            for col in 0x00..=0x0F {
                let register = row + col;
                match self.read_sm(register) {
                    Ok(byte) => print!(" {:?} ", byte),
                    Err(e) => {
                        println!("Error reading row byte at register: {}", register);
                        println!("{e}");
                        return;
                    }
                }
            }
            println!(); // New line at the end of each row
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
