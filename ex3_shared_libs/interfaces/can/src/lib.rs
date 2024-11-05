/*
Written by: Amar

The CAN module uses the socketCAN utility
from Linux.
*/

use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket, StandardId};
use std::io::Error;

struct CanInterface {
    socket: CanSocket,
}

impl CanInterface {
    pub fn new(socket_name: &str) -> Result<CanInterface, Error> {
        let socket = CanSocket::open(socket_name)?;
        let can_interface = CanInterface { socket };

        Ok(can_interface)
    }

    pub fn send(&mut self, frame: &CanFrame) -> Result<(), Error> {
        self.socket.write_frame(frame)
    }

    pub fn recv(&mut self, timeout: std::time::Duration) -> Result<CanFrame, Error> {
        self.socket.read_frame_timeout(timeout)
    }
}

pub fn make_standard_frame(data: &[u8], id: u16) -> Option<CanFrame> {
    let standard_id = match StandardId::new(id) {
        Some(id) => id,
        None => return None,
    };
    let frame = match CanFrame::new(standard_id, data) {
        Some(f) => f,
        None => return None,
    };

    Some(frame)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_vcan0() {
        let mut socket1 = CanInterface::new("vcan0").unwrap();
        let mut socket2 = CanInterface::new("vcan0").unwrap();

        let data: &[u8; 3] = &[1, 2, 3];
        let id = 0x1FF;
        let msg = make_standard_frame(data, id).unwrap();

        let _ = socket1.send(&msg).unwrap();
        let recieved = socket2.recv(Duration::from_secs(1)).unwrap();

        assert_eq!(recieved.data(), msg.data());
        assert_eq!(recieved.id(), msg.id());
    }
}
