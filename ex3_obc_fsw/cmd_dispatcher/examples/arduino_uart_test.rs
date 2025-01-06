use std::os::fd::AsFd;
use interface::{uart, Interface};
use nix::poll::{poll, PollFd, PollFlags, PollTimeout};

fn main() {
    let mut arduino = uart::UartInterface::new("/dev/ttyACM0", None);
    let mut read_buff: [u8; 36] = [0; 36];
    let polling_fd = arduino.fd.try_clone().unwrap();
    let pfd = PollFd::new(polling_fd.as_fd(), PollFlags::POLLIN);
    let mut fds = [pfd];

    loop {
        let _ret = match poll(&mut fds, PollTimeout::NONE) {
            Ok(n) => {
                n
            },
            Err(e) => {
                println!("Error occurred while polling: {e}");
                continue;
            }
        };
        let bytes_to_read = match arduino.available_to_read() {
            Ok(n) => n,
            Err(e) => {
                println!("Could not find bytes available to read: {e}");
                continue;
            }

        };
        if bytes_to_read >= 36 {
            match arduino.read(&mut read_buff) {
                Ok(bytes) => {
                    println!("Bytes Read: {}", bytes );
                }
                Err(e) => {
                    println!("Error reading from arduino: {e}");
                    continue;
                }
            }
            println!("{:?}", read_buff);
            let _ = arduino.flush_input();
        }
    }
}

