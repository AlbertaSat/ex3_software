/*
Written by Kaaden RumanCam and Ron Unrau
Summer 2024

...

TODO - If connection is lost with an interface, attempt to reconnect every 5 seconds
TOOD - Figure out way to use polymorphism and have the interfaces be configurable at runtime (i.e. TCP, UART, etc.)
TODO - Get state variables from a state manager (channels?) upon instantiation and update them as needed.
TODO - Setup a way to handle opcodes from messages passed to the handler

*/

use std::io::Error;
use ipc::{poll_ipc_clients, IpcClient, IPC_BUFFER_SIZE};
use log::{debug, trace, warn};
use message_structure::*;

struct ShellHandler {
    msg_dispatcher_interface: Option<IpcClient>, // For communcation with other FSW components [internal to OBC]
}

impl ShellHandler {
    pub fn new(
        msg_dispatcher_interface: Result<IpcClient, std::io::Error>,
    ) -> ShellHandler {
        if msg_dispatcher_interface.is_err() {
            warn!(
                "Error creating dispatcher interface: {:?}",
                msg_dispatcher_interface.as_ref().err().unwrap()
            );
        }
        ShellHandler {
            msg_dispatcher_interface: msg_dispatcher_interface.ok(),
        }
    }

    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) -> std::io::Result<()> {
        // Poll for messages
        loop {
            let msg_dispatcher_interface = self.msg_dispatcher_interface.as_mut().expect("msg_dispatcher_interface has value of None");

            let mut clients = vec![
                msg_dispatcher_interface,
            ];
            poll_ipc_clients(&mut clients)?;

            // Handling the bulk message dispatcher interface
            let msg_dispatcher_interface = self.msg_dispatcher_interface.as_ref().unwrap();
            if msg_dispatcher_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
                let recv_msg: Msg = deserialize_msg(&msg_dispatcher_interface.buffer).unwrap();
                debug!("Received and deserialized msg");
                self.handle_msg(recv_msg)?;
            }
        }
    }
    //TODO - After receiving the message, send a response back to the dispatcher ??

    fn handle_msg(&mut self, msg: Msg) -> Result<(), Error> {
        self.msg_dispatcher_interface.as_mut().unwrap().clear_buffer();
        println!("SHELL msg opcode: {} {:?}", msg.header.op_code, msg.msg_body);
        Ok(())
    }
}

fn main() {
    trace!("Starting Shell Handler...");

    // Create Unix domain socket interface for to talk to message dispatcher
    let msg_dispatcher_interface = IpcClient::new("shell_handler".to_string());

    let mut shell_handler = ShellHandler::new(msg_dispatcher_interface);

    let _ = shell_handler.run();
}
