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
use std::process::Command;

use common::component_ids::ComponentIds::{GS, SHELL};
use common::constants::DONWLINK_MSG_BODY_SIZE;
use ipc::{ipc_write, poll_ipc_server_sockets, IpcClient, IpcServer, IPC_BUFFER_SIZE};
use log::{debug, trace, warn};
use logging::*;
use message_structure::*;

struct ShellHandler {
    msg_dispatcher_interface: Option<IpcServer>, // For communcation with other FSW components [internal to OBC]
    gs_interface: Option<IpcClient>, // To send messages to the GS through the coms_handler
}

impl ShellHandler {
    pub fn new(
        msg_dispatcher_interface: Result<IpcServer, std::io::Error>,
        gs_interface: Result<IpcClient, std::io::Error>,
    ) -> ShellHandler {
        if msg_dispatcher_interface.is_err() {
            warn!(
                "Error creating dispatcher interface: {:?}",
                msg_dispatcher_interface.as_ref().err().unwrap()
            );
        }
        if gs_interface.is_err() {
            warn!(
                "Error creating gs interface: {:?}",
                gs_interface.as_ref().err().unwrap()
            );
        }
        ShellHandler {
            msg_dispatcher_interface: msg_dispatcher_interface.ok(),
            gs_interface: gs_interface.ok(),
        }
    }

    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) -> std::io::Result<()> {
        // Poll for messages
        loop {
            // First, take the Option<IpcClient> out of `self.dispatcher_interface`
            // This consumes the Option, so you can work with the owned IpcClient
            let msg_dispatcher_interface = self.msg_dispatcher_interface.take().expect("Cmd_Disp has value of None");

            // Create a mutable Option<IpcClient> so its lifetime persists
            let mut msg_dispatcher_interface_option = Some(msg_dispatcher_interface);

            // Now you can borrow this mutable option and place it in the vector
            let mut server: Vec<&mut Option<IpcServer>> = vec![
                &mut msg_dispatcher_interface_option,
            ];

            poll_ipc_server_sockets(&mut server);

            // restore the value back into `self.dispatcher_interface` after polling. May have been mutated
            self.msg_dispatcher_interface = msg_dispatcher_interface_option;

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

        trace!("SHELL msg opcode: {} {:?} = {}", msg.header.op_code, msg.msg_body, String::from_utf8(msg.msg_body.clone()).unwrap());

        let body = String::from_utf8(msg.msg_body).unwrap();

        match Command::new("bash").arg("-c").arg(body).output() {
            Ok(out) => {
                trace!("command outputted: {}", String::from_utf8(out.stdout.clone()).unwrap());

                for chunk in out.stdout.chunks(DONWLINK_MSG_BODY_SIZE) {
                    let msg = Msg::new(MsgType::Cmd as u8, 0, GS as u8, SHELL as u8, 0, chunk.to_vec());
                    if let Some(gs_resp_interface) = &self.gs_interface {
                        let _ = ipc_write(&gs_resp_interface.fd, &serialize_msg(&msg)?);
                    } else {
                        debug!("Response not sent to gs. IPC interface not created");
                    }
                }
            },
            Err(e) => {
                trace!("command failed: {e}");
            },
        };

        Ok(())
    }
}

fn main() {
    let log_path = "ex3_obc_fsw/handlers/shell_handler/logs";
    init_logger(log_path);

    trace!("Starting Shell Handler...");

    // Create Unix domain socket interface for to talk to message dispatcher
    let msg_dispatcher_interface = IpcServer::new("SHELL".to_string());

    let gs_interface = IpcClient::new("gs_non_bulk".to_string());

    let mut shell_handler = ShellHandler::new(msg_dispatcher_interface, gs_interface);

    let _ = shell_handler.run();
}
