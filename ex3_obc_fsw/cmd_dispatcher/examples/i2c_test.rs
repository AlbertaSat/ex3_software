use interface::{i2c::*, Interface};
use std::{process::exit, thread::sleep, time};
// Enter I2C bus path here
const BUS_PATH: &str = "/dev/i2c-0";

// Enter Device Address here
// Need to double check this as it depends on the device
const ADDRESS: u16 = 0x48;

//default config for ads1100
#[allow(dead_code)]
const CONFIG: u8 = 0x8C;

// two bytes we need to send to device to do a general call reset if neccesary
#[allow(dead_code)]
const GENERAL_CALL_RESET: [u8; 2] = [0x00, 0x06];

const SLEEP_TIME: time::Duration = time::Duration::from_millis(50);

fn main() {
    // Initialize the I2c interafce with lux meter
    let mut lux_meter = I2cDeviceInterface::new(BUS_PATH, ADDRESS).unwrap();

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
                exit(-1);
            }
        }
    }
}
