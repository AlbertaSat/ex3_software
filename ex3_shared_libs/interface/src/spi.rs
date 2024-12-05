use super::Interface;
use spidev::*;
use std::io::{Read, Write, Error};

pub struct SpiInterface {
    device: Spidev,
    path: String,
}

impl Interface for SpiInterface {
    // Sends the bytes from the user provided buffer to the spi device, returns the number of bytes
    // written
    fn send(&mut self, buff: &[u8]) -> Result<usize, Error> {
        let bytes_written = self.device.write(buff)?;
        Ok(bytes_written)
    }

    // Reads bytes from the spi into the buffer provided by the user, returns the number of
    // bytes read
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, Error> {
        let bytes_read = self.device.read(buff)?;
        Ok(bytes_read)
    }
}

impl SpiInterface {
    // Function to create SpiInterface
    pub fn new(path: &str, options: Option<SpidevOptions>) -> Result<Self, Error> {
        let mut spi_device = Spidev::open(path)?;
        let spi_config = match options {
            // If user provides a SpidevOptions struct to configure spi then we create device with
            // config
            Some(config) => config,
            // User did not select any config so we use the following default options
            None => {
                SpidevOptions::new()
                    .bits_per_word(8)
                    .max_speed_hz(5000)
                    .lsb_first(false)
                    .mode(SpiModeFlags::SPI_MODE_0)
                    .build()
            }
        };
        spi_device.configure(&spi_config)?;
        Ok(SpiInterface {device: spi_device, path: path.to_string()})
    }
    // getter for device path
    pub fn get_dev_path(&mut self) -> String {
        self.path.clone()
    }
    // Flushes the output stream
    pub fn flush_out_buff(&mut self) -> Result<(), Error> {
        self.device.flush()
    }
}
