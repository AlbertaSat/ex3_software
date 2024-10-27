use std::{thread, time};
use uart::UARTInterface;
fn main() {
    let mut arduino_serial = UARTInterface::new("/dev/ttyACM0", 9600).unwrap();

    let msg = [0x01, 0x02, 0x03, 0x04];

    loop {
        match arduino_serial.write_raw_bytes(&msg) {
            Ok(len) => {
                println!("wrote {} bytes to arduino", len);
            }
            Err(e) => {
                println!("Error writing to arduino!!!");
                println!("{}", e);
                break;
            }
        };
        thread::sleep(time::Duration::from_millis(500));
    }
}
