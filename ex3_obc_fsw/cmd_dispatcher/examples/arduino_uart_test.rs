use std::io::{self, Read};
use std::panic::panic_any;
use std::sync::mpsc;
use std::thread;
use interface::{uart::UARTInterface, Interface};

fn main() {
    let mut arduino_serial = UARTInterface::new("/dev/ttyACM2", 9600).unwrap();

    let mut buffer = [0; 36].to_vec();
    let mut data_collection = false;

    let user_input = input_service();

    loop {
        if arduino_serial.available_to_read().unwrap() == 36 {
            match arduino_serial.read(&mut buffer) {
                Ok(0) => println!("No bytes to read"), // No bytes to read
                Ok(_n) => {
                    println!("{:?}", buffer);
                    arduino_serial
                        .clear_input_buffer()
                        .expect("Failed to clear input buffer");
                }
                Err(e) => {
                    println!("Error reading from simulated dfgm: {e}");
                    break;
                }
            }
        }
        match user_input.try_recv() {
            Ok(_) => {
                println!("Toggling data collection");
                data_collection = !data_collection;
                if data_collection {
                    match arduino_serial.send(&[1]) {
                        Ok(_) => {
                            println!("Data collection toggled on.");
                            arduino_serial
                                .clear_input_buffer()
                                .expect("Failed to clear input buffer")
                        }
                        Err(e) => {
                            println!("Error toggling data collection: {e}");
                        }
                    }
                } else {
                    match arduino_serial.send(&[0]) {
                        Ok(_) => {
                            println!("Data collection toggled off.");
                        }
                        Err(e) => {
                            println!("Error toggling data collection: {e}");
                        }
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => (),
            Err(e) => {
                println!("Stopping due to: {e}");
                break;
            }
        }
    }
}

fn input_service() -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut buffer = [0; 32];
        loop {
            // Block awaiting any user input
            match io::stdin().read(&mut buffer) {
                Ok(0) => {
                    drop(tx); // EOF, drop the channel and stop the thread
                    break;
                }
                Ok(_bytes_read) => tx.send(()).unwrap(), // Signal main to stop collection
                Err(e) => panic_any(e),
            }
        }
    });

    rx
}
