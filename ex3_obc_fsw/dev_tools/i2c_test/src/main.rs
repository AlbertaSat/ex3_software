use core::panicking::panic;
use i2c::*;
use std::{thread::sleep, time};
// Enter I2C bus path here
const BUS_PATH: &str = "/dev/i2c-___";

// Enter Device Address here
// Need to double check this as it depends on the device
const ADDRESS: u16 = 0x48;

//default config for ads1100
const CONFIG: u8 = 0x8C;

// two bytes we need to send to device to do a general call reset if neccesary
const GENERAL_CALL_RESET: [u8; 2] = [0x00, 0x06];

const SLEEP_TIME: time::Duration = time::Duration::from_millis(50);

fn main() {
    // Initialize the I2c interafce with lux meter
    let mut lux_meter = I2cDeviceInterface::new(BUS_PATH, ADDRESS).unwrap();

    // Perform general call reset
    match lux_meter.send(&GENERAL_CALL_RESET) {
        Ok(val) => {
            println!("Sent {}, bytes.", val);
            println!("Successfully reset device.");
        }
        Err(e) => {
            println!("Failed to send general call reset to device.");
            println!("{}", e);
        }
    };
    sleep(SLEEP_TIME);

    // Write Confiuration Byte
    // Do not need to set config since its already set at default, this code is here incase we want
    // to change config from default
    match lux_meter.send(&[CONFIG]) {
        Ok(_) => {
            println!("Successfully sent configuration byte.");
        }
        Err(e) => {
            println!("Writing configuration to the ADS failed.");
            println!("{}", e);
        }
    };

    sleep(SLEEP_TIME);

    // Continually read from the device
    let mut buffer: [u8; 2] = [0; 2];
    loop {
        match lux_meter.read(&mut buffer) {
            Ok(_) => {
                let result: i16 = (buffer[0] as i16) << 8 | buffer[1] as i16;
                println!("{}", result);
                sleep(SLEEP_TIME); // Sleep a small amount of time after each interaction
            }
            Err(e) => {
                println!("Error reading from device.");
                println!("{}", e);
                panic!();
            }
        }
    }
}
