use std::{thread, time};
use uart::UARTInterface;
fn main() {
    let mut arduino_serial = UARTInterface::new("/dev/ttyACM0", 9600).unwrap();

    let mut buffer = [0; 50].to_vec();

    loop {
        match arduino_serial.read_raw_bytes(&mut buffer) {
            Ok(_) => {
                println!("Got message: {:?}", &buffer);
            }
            Err(e) => {
                println!("Error reading from arduino!!!");
                println!("{}", e);
                break;
            }
        };
        buffer.fill(0);
        thread::sleep(time::Duration::from_millis(500));
    }
}
