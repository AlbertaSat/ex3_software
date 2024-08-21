/*
Written by Devin Headrick
Summer 2024

This library is to allow OBC FSW components to communicate with devices using CAN (Controller Area Network) bus protocol.
Particularly this is to interact with the NanoAvionics EPS, which uses CSP protocol over CAN bus to communicate with the OBC.

For the OBC FSW this is an odd usage of CAN as we are using it as a point to point communication protocol, while its intended to
be used as a broadcast on a bus shared by many devices.


Can standard used by the NanoAvionics EPS [from NanoAvionics EPS ICD Revision 8, page 108]
- Version 2.0B
- SP set to 87.5%
- SJW = 1
- bus speed 1000kbps

Online Calculator for fidning CAN timing parameters:
http://www.bittiming.can-wiki.info/


Notes on CAN:
Linux uses socketCAN protocol - which uses berkley socket API, linux network stack, and implements CAN device drivers as network interfaces.

This crate was updated 10 months ago: https://github.com/socketcan-rs/socketcan-rs

A 'virtual CAN' device can be created easily for testing.

'frames' - messages - are multicast.
Each frame consists of an ID, a payload of up to 8 bytes.

*/

use socketcan::{CanFrame, CanSocket, Frame, Socket};
use std::time::Duration;

const CAN_READ_TIMEOUT_MS: u64 = 100;

fn convert_frame_to_string<F: Frame>(frame: &F) -> String {
    let id = frame.raw_id();
    let data_string = frame
        .data()
        .iter()
        .fold(String::from(""), |a, b| format!("{} {:02X}", a, b));
    format!("{:08X}  [{}] {}", id, frame.dlc(), data_string)
}

struct CanInterface {
    socket_interface_name: String,
    socket: CanSocket,
}

impl CanInterface {
    pub fn new(socket_interface_name: String) -> Result<Self, std::io::Error> {
        let sock = CanSocket::open(&socket_interface_name);
        if let Ok(sock) = sock {
            let can_interface = CanInterface {
                socket_interface_name: socket_interface_name,
                socket: sock,
            };
            return Ok(can_interface); 
        }
        else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "CAN Error creating can interface"));
        }
    }

    pub fn reconnect(socket_interface_name: String) {
        //TODO - implement this so reconnection attempt can happen without creating new instance
    }

    fn read_frame_w_timeout(&self) -> Result<CanFrame, std::io::Error> {
        let frame_read = self.socket.read_frame_timeout(Duration::from_millis(CAN_READ_TIMEOUT_MS));
        if let Ok(frame) = frame_read {
            let frame_str = convert_frame_to_string(&frame);
            println!("Frame read: {:?}", frame_str);
            Ok(frame)
        }
        else {
            println!("Invalid frame read");
            Err(std::io::Error::new(std::io::ErrorKind::Other, "CAN Frame Read error"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //TODO - implement this 
    #[test]
    fn test_with_vcan0(){
        // First be sure to create virtual can interface vcan0 - using socketCAN driver for linux

        let can_interface = CanInterface::new("vcan0".to_string()); 
        assert!(can_interface.is_ok());
    }

    #[test] 
    fn test_frame_read_w_timeout(){
        //TODO 
    }
}
