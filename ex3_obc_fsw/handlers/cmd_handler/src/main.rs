/*
Written By Devin Headrick and Rowan Rasmusson
Summer 2024

TODO - If connection is lost with an interface, attempt to reconnect every 5 seconds
TOOD - Figure out way to use polymorphism and have the interfaces be configurable at runtime (i.e. TCP, UART, etc.)
TODO - Get state variables from a state manager (channels?) upon instantiation and update them as needed.
TODO - Setup a way to handle opcodes from messages passed to the handler

*/

use std::io::Error;
use ipc::{poll_ipc_clients, IpcClient, IPC_BUFFER_SIZE};

use message_structure::*;
use logging::*;
use log::{debug, trace, warn};

/// Interfaces are option types in case they are not properly created upon running this handler, so the program does not panic
struct CmdHandler {
    // For communcation with other FSW components [internal to OBC]
    msg_dispatcher_interface: Option<IpcClient>,
}

impl CmdHandler {
    pub fn new(
        msg_dispatcher_interface: Result<IpcClient, std::io::Error>,
    ) -> CmdHandler {
        if msg_dispatcher_interface.is_err() {
            warn!("Error creating dispatcher interface: {:?}",
                  msg_dispatcher_interface.as_ref().err().unwrap()
            );
        }

        CmdHandler {
            msg_dispatcher_interface: msg_dispatcher_interface.ok(),
        }
    }

    fn handle_msg(&mut self, msg: Msg) -> Result<(), Error> {
        self.msg_dispatcher_interface.as_mut().unwrap().clear_buffer();
        trace!("CMD msg opcode: {}", msg.header.op_code);
        Ok(())
    }

    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) -> std::io::Result<()> {
        // Read and poll for input for a message
        loop {
            // Borrowing the dispatcher interfaces
            let msg_dispatcher_interface = self.msg_dispatcher_interface.as_mut().expect("Cmd_Msg_Disp has value of None");

            let mut clients = vec![
                msg_dispatcher_interface,
            ];
            poll_ipc_clients(&mut clients)?;
            
            // Handling the bulk message dispatcher interface
            if let Some(cmd_msg_dispatcher) = self.msg_dispatcher_interface.as_mut() {
                if cmd_msg_dispatcher.buffer != [0u8; IPC_BUFFER_SIZE] {
                    let recv_msg: Msg = deserialize_msg(&cmd_msg_dispatcher.buffer).unwrap();
                    debug!("Received and deserialized msg");
                    self.handle_msg(recv_msg)?;
                }
            }
        }
    }
    //TODO - After receiving the message, send a response back to the dispatcher ??
}

fn main() -> Result<(), Error> {
    trace!("Starting CMD Handler...");

    init_logger("ex3_obc_fsw/handlers/cmd_handler/logs");

    //Create Unix domain socket interface for CMD handler to talk to message dispatcher
    let msg_dispatcher_interface = IpcClient::new("cmd_handler".to_string());

    let mut cmd_handler = CmdHandler::new(msg_dispatcher_interface);

    cmd_handler.run()
}
