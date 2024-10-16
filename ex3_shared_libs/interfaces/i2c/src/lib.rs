extern crate i2cdev;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::io::{Error as IOError, Write};

// Interface to be implemented by all external interfaces
pub trait Interface {
    // send data as bytes to the interface as a shared byte type slice.
    // Returns the number of bytes sent
    fn send(&mut self, data: &[u8]) -> Result<usize, LinuxI2CError>;

    // Read byte data from the interfaace into a byte slice buffer.
    // Returns number of bytes read
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, IOError>;
}

// Structure for I2C Interface, i2c is the actual interface while the slave address is where data
// is going to be written and/or read from
pub struct I2cDeviceInterface {
    device: LinuxI2CDevice,
    bus_path: String,
    client_address: u16,
}

impl Interface for I2cDeviceInterface {
    // Used to indicate what communication protocol the device on the I2C bus reimpl Interface for I2cDeviceInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize, LinuxI2CError> {
        // Send data to the slave using the i2c interface
        // I have no idea what the write function actually does. it doesnt need an address or
        // anything which is weird. like which registers is it writing to?

        self.device.write(data)?;
        Ok(data.len())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, IOError> {
        // read data from the slave using the i2c interface
        self.device.read(buffer)?;
        Ok(buffer.len())
    }
}

impl I2cDeviceInterface {
    pub fn new(path: &str, client_address: u16) -> Result<I2cDeviceInterface, LinuxI2CError> {
        // Initalize i2c interface with path and then pass slave address
        let device = LinuxI2CDevice::new(path, client_address)?;
        Ok(I2cDeviceInterface {
            device,
            bus_path: path.to_string(),
            client_address,
        })
    }

    fn send_byte(&mut self, register: u8, byte: u8) -> Result<(), LinuxI2CError> {
        // Send data to the slave using the i2c interface
        self.device.smbus_write_byte_data(register, byte)?;
        Ok(())
    }

    fn read_byte(&mut self, address: u8) -> Result<u8, LinuxI2CError> {
        // read data from the slave using the i2c interface
        self.device.smbus_read_byte_data(address)
    }

    fn print_device_registers(&mut self) {
        println!("     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f");

        for row in (0x00..=0xFF).step_by(16) {
            print!("{:02X}: ", row);

            for col in 0x00..=0x0F {
                let register = row + col;
                match self.read_byte(register) {
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
    use core::panic;

    const BUS_PATH: &str = "/dev/i2c-16";
    const CLIENT_ADDRESS: u16 = 0x1C;

    #[test]
    fn test_sm_bus_read_and_write() {
        let mut device = I2cDeviceInterface::new(BUS_PATH, CLIENT_ADDRESS).unwrap();

        match device.send_byte(0xFF, 0x66) {
            Ok(_) => {
                println!(
                    "Successfully wrote to device on bus {} at address 0x{:X}",
                    device.bus_path, device.client_address
                );
            }

            Err(e) => {
                println!(
                    "Error writing to device on bus {} at 0x{:X}",
                    device.bus_path, device.client_address
                );
                println!("{e}");
                panic!();
            }
        }

        match device.read_byte(0xff) {
            Ok(byte) => {
                println!(
                    "Got byte: 0x{:X}, from device on bus {} at address 0x{:X}",
                    byte, device.bus_path, device.client_address
                );
            }

            Err(e) => {
                println!(
                    "Error Reading from device on bus {} at address 0x{:X}",
                    device.bus_path, device.client_address
                );
                println!("{e}");
                panic!();
            }
        }
    }

    #[test]
    fn test_read() {
        let mut device = I2cDeviceInterface::new(BUS_PATH, CLIENT_ADDRESS).unwrap();

        let mut buffer: [u8; 30] = [0; 30];
        match device.read(&mut buffer) {
            Ok(n) => {
                println!(
                    "Read {} bytes from device at address {}",
                    n, device.client_address
                );
                println!("Buffer Contents:\n{:?}", &buffer)
            }
            Err(e) => {
                println!(
                    "Error occured when reading from device at address {}",
                    device.client_address
                );
                println!("{e}");
                panic!();
            }
        }
    }
}
