/*
Written by Devin Headrick
Summer 2024

This is a short - fat implementation of the EPS handler designed to interact directly with the Nanoavionics EPS hardware.

TODO
    - Implement re-connect attempts for interfaces if any interaction with them fails
*/

mod commands;
mod enums;
mod housekeeping;

use housekeeping::Housekeeping;

use common::opcodes;
use ipc::{ipc_write, poll_ipc_clients, IpcClient, IPC_BUFFER_SIZE};
use message_structure::{deserialize_msg, serialize_msg, Msg, MsgType};
use tcp_interface::{Interface, TcpInterface};

const EPS_BUFFER_SIZE: usize = 1024;

struct Handler {
    peripheral_interface: Option<TcpInterface>,
    dispatcher_interface: Option<IpcClient>, // interfaces are options in case they fail to init, or connected dropped and must be reconnected, without panic handler
    housekeeping: Housekeeping,
}

impl Handler {
    fn build_handler_default(
        peripheral_interface: Option<TcpInterface>,
        dispatcher_interface: Option<IpcClient>,
    ) -> Handler {
        Handler {
            peripheral_interface: peripheral_interface,
            dispatcher_interface: dispatcher_interface,
            housekeeping: Housekeeping::new_default(),
        }
    }

    fn handle_msg_from_dispatcher(&self, msg: Msg) {
        let msg_opcode_enum = common::opcodes::EPS::from(msg.header.op_code);

        match msg_opcode_enum {
            opcodes::EPS::Ping => {
                //TODO - implement ping
                println!("Msg to Ping EPS received from Dispatcher");
            }
            _ => {
                //TODO - implement default case
            }
        }
    }

    /// Loop indefinitely, polling interfaces for inputs and handling them
    fn run(&mut self) -> Result<(), std::io::Error> {
        //TODO - Get rid of these unwraps and handle errors with interfaces without panic

        loop {
            let mut ipc_interface_vec = vec![self.dispatcher_interface.as_mut().unwrap()];
            // let eps_interface = self.peripheral_interface.as_mut().unwrap();
            // let mut peripheral_read_buf = [0; EPS_BUFFER_SIZE];
            // -------------------------------------------------------------------------------------

            let ipc_bytes_read = poll_ipc_clients(&mut ipc_interface_vec)?;

            // TODO - (maybe??) modify the poll fxn to also return the index of the interface that had a message, so we can match it to the correct interface in the vec
            if let Some(dispatcher_interface) = self.dispatcher_interface.as_mut() {
                if ipc_bytes_read > 0 {
                    let read_bytes_raw = dispatcher_interface.read_buffer();
                    let read_msg = deserialize_msg(&read_bytes_raw)?;
                    println!("dispatcher bytes read: {:?}", read_msg);
                    self.handle_msg_from_dispatcher(read_msg);
                }
            }

            // let eps_bytes_read = eps_interface.read(&mut peripheral_read_buf)?;
            // if eps_bytes_read > 0 {
            //     let read_msg = deserialize_msg(&peripheral_read_buf);
            //     println!("peripheral bytes read: {:?}", read_msg);
            // }

            //TODO - Poll peripheral interfaces for messages
        }
    }
}

fn setup_interfaces() -> (Option<TcpInterface>, Option<IpcClient>) {
    let mut peripheral_interface = None;
    let peripheral_interface_result =
        TcpInterface::new_client("127.0.0.1".to_string(), common::ports::SIM_EPS_PORT);

    match peripheral_interface_result {
        Ok(interface_returned) => {
            println!("Connected to peripheral interface");
            peripheral_interface = Some(interface_returned);
        }
        Err(e) => {
            println!("Error connecting to peripheral interface: {}", e);
        }
    }

    let mut dispatcher_interface = None;
    let dispatcher_interface_result = IpcClient::new("eps_handler".to_string());

    match dispatcher_interface_result {
        Ok(interface_returned) => {
            println!("Connected to dispatcher interface");
            dispatcher_interface = Some(interface_returned);
        }
        Err(e) => {
            println!("Error connecting to dispatcher interface: {}", e);
        }
    }

    (peripheral_interface, dispatcher_interface)
}

fn main() {
    let interfaces = setup_interfaces();
    println!("Interfaces setup");
    let mut eps_handler = Handler::build_handler_default(interfaces.0, interfaces.1);
    eps_handler.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    //TODO - set this up
    #[test]
    fn test_handler_build_interfaces_good() {
        //Create instance of 'ipc server dev tool' for interacting with handler via ips in place of message dispatcher
        // Create instance of simulated eps subsystem for testing
        // Setup interfaces knowing that the above two are good
        let interfaces = setup_interfaces();
        let handler = Handler::build_handler_default(None, None);
    }

    #[test]
    fn test_handler_build_interfaces_none() {
        let handler = Handler::build_handler_default(None, None);
    }
}
