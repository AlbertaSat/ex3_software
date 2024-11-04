/// Written by: Amar
///
/// The CAN module uses the socketCAN utility
/// from Linux.
///
use socketcan::{CanFrame, CanSocket, Frame, Socket};

pub trait Interface {
    pub fn recv();
    pub fn send();
}

pub struct CanInterface {
    socket: CanSocket,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
