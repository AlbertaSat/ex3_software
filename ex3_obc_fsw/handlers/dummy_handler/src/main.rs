/*
written by Devin Headrick
summer 2024

TODO
    - wait to execute a command message until the current one has been handled
    - setup handler to re-attempt connection to subsystem if it fails or connection drops
*/

use common::component_ids::ComponentIds;
use common::opcodes;
use common::ports;
use core::str;
use ipc::{ipc_write, poll_ipc_clients, IpcClient, IPC_BUFFER_SIZE};
use message_structure::{deserialize_msg, serialize_msg, Msg, MsgType};
use tcp_interface::{Interface, TcpInterface};

const DUMMY_PERIPHERAL_MAX_MSG_SIZE_BYTES: u8 = 128; //largest size of a data packet from dummy subsystem sim
const DUMMY_MSG_DELIMETER: &str = ":";

/// THIS IS HOW WE DEFINE WHAT A MSG SPECIFIC FOR THE 'DUMMY' PERIPHERAL LOOKS LIKE
/// (i.e. each peripheral will have its own struct like this based on the unique 'protocol' outlined in its user manual)
struct DummyPeripheralMsg {
    command_string: String,
    command_data: Option<String>,
}

/// Contains function for converting between Msg structs used by the FSW, and the peripheral specific message struct
impl DummyPeripheralMsg {
    fn format_peripheral_msg_outgoing(&self) -> String {
        match &self.command_data {
            Some(data) => format!("{}{}{}", self.command_string, DUMMY_MSG_DELIMETER, data),
            None => self.command_string.clone(),
        }
    }
}

/// Create peripheral message from opcode and data of received command message
impl From<Msg> for DummyPeripheralMsg {
    fn from(value: Msg) -> Self {
        let fxn_from_corresponding_opcode = value.header.op_code.into();
        match fxn_from_corresponding_opcode {
            opcodes::DUMMY::SetDummyVariable => {
                let new_dummy_var_value = value.msg_body[0];
                Self {
                    command_string: "SET_DUMMY_VAR".to_string(),
                    command_data: Some(new_dummy_var_value.to_string()),
                }
            }
            opcodes::DUMMY::GetDummyVariable => Self {
                command_string: "GET_DUMMY_VAR".to_string(),
                command_data: None,
            },
            _ => {
                eprintln!("Invalid opcode: {}", fxn_from_corresponding_opcode as u8);
                Self {
                    command_string: "INVALID".to_string(),
                    command_data: None,
                }
            }
        }
    }
}

struct DummyHandler {
    peripheral_interface: TcpInterface,
    dispatcher_interface: IpcClient,
    awaiting_peripheral_response: bool,
    cmd_msg_buffer: Vec<Msg>,
}

impl DummyHandler {
    fn new(peripheral_interface: TcpInterface, dispatcher_interface: IpcClient) -> Self {
        Self {
            peripheral_interface,
            dispatcher_interface,
            awaiting_peripheral_response: false, //This is set true after sending the peripheral a message and waiting for a response
            cmd_msg_buffer: Vec::new(), //Msgs are added here as they are recv'd from dispatcher, and removed after they are handled
        }
    }

    // ----------------------------------------------------------------------------------------------------------------------
    /*
     * Here goes functions related uniquely to the subsystem the handler is associated with (i.e. decryption for gs handler, )
     */
    fn set_dummy_subsystem_variable(&mut self, msg_body: Vec<u8>) -> Result<usize, std::io::Error> {
        println!(
            "Set dummy subsystem variable called with value: {:?}",
            msg_body[0]
        );
        let set_dummy_var_msg = DummyPeripheralMsg::from(self.cmd_msg_buffer[0].clone());
        let set_dummy_var_msg_string = set_dummy_var_msg.format_peripheral_msg_outgoing();
        self.peripheral_interface
            .send(set_dummy_var_msg_string.as_bytes())
    }

    fn get_dummy_subsystem_variable(&mut self) -> Result<usize, std::io::Error> {
        println!("Get dummy subsystem variable called");
        let get_dummy_var_msg = DummyPeripheralMsg::from(self.cmd_msg_buffer[0].clone());
        let get_dummy_var_msg_string = get_dummy_var_msg.format_peripheral_msg_outgoing();
        self.peripheral_interface
            .send(get_dummy_var_msg_string.as_bytes())
    }

    // ----------------------------------------------------------------------------------------------------------------------
    /*
     * Here we handle messages received from the subsystems peripheral
     */
    fn handle_msg_from_peripheral(&mut self, peripheral_msg_bytes: Vec<u8>) {
        println!(
            "received message from dummy subsystem: {:?}",
            peripheral_msg_bytes
        );

        //Convert recevied bytes from peripheral into its associated 'peripheral msg' struct
        let peripheral_response_msg_string = str::from_utf8(&peripheral_msg_bytes).unwrap().trim();
        println!(
            "Recevied peripheral message string: {}",
            peripheral_response_msg_string
        );

        // Build an 'ACK' message to send back to GS to acknowledge the command was received, containing requested data
        let id_of_msg_responding_to = self.cmd_msg_buffer[0].header.msg_id;
        let opcode_of_msg_responding_to = self.cmd_msg_buffer[0].header.op_code;
        let ack_msg_body = format!("ACK:{}", peripheral_response_msg_string);
        let cmd_msg_response_ack = Msg::new(
            MsgType::Ack as u8,
            id_of_msg_responding_to,
            ComponentIds::GS.into(), //Swap destination and source for downlink
            ComponentIds::DUMMY.into(),
            opcode_of_msg_responding_to,
            ack_msg_body.as_bytes().to_vec(),
        );

        // Send the message to the dispatcher for downlink back to GS
        let _ = ipc_write(
            self.dispatcher_interface.fd,
            serialize_msg(&cmd_msg_response_ack).unwrap().as_slice(),
        );

        self.cmd_msg_buffer.remove(0);
    }

    // ----------------------------------------------------------------------------------------------------------------------
    fn handle_msg_from_dispatcher(&mut self, msg: Msg) {
        // Use the 'from<u8>' trait implemented for the subsystems associated opcode enum
        let opcode = opcodes::DUMMY::from(msg.header.op_code);
        self.cmd_msg_buffer.push(msg.clone());
        let msg_handle_res = match opcode {
            opcodes::DUMMY::SetDummyVariable => self.set_dummy_subsystem_variable(msg.msg_body),
            opcodes::DUMMY::GetDummyVariable => self.get_dummy_subsystem_variable(),
            // _ => println!("invalid opcode received"),
        };

        match msg_handle_res {
            Ok(_) => {
                self.awaiting_peripheral_response = true;
            }
            Err(e) => {
                //TODO - emit 'nack'
                println!("error handling message from dispatcher: {:?}", e);
            }
        }
    }

    /// Main loop for running the handler
    /// This is where we listen (read) incomming messages from the subsystem perihpheral and ipc interfaces, and call the appropriate functions to handle them
    pub fn run(&mut self) {
        loop {
            {

                // Poll the 

                let mut ipc_client_vec = vec![&mut self.dispatcher_interface];
                poll_ipc_clients(&mut ipc_client_vec).unwrap();

                let dispatcher = &mut self.dispatcher_interface;
                if dispatcher.buffer != [0u8; IPC_BUFFER_SIZE] {
                    println!(
                        "received message from ipc interface: {:?}",
                        dispatcher.buffer
                    );

                    let deserialized_msg_res = deserialize_msg(&dispatcher.buffer);
                    // Handle the deserialized message in a separate scope
                    let command_msg = match deserialized_msg_res {
                        Ok(deserialized_msg) => Some(deserialized_msg),
                        Err(e) => {
                            println!("error deserializing message: {:?}", e);
                            // Optionally send a failure acknowledgement here
                            None
                        }
                    };

                    dispatcher.clear_buffer();

                    if let Some(msg) = command_msg {
                        self.handle_msg_from_dispatcher(msg);
                    }
                }
            }
            {
                let mut dummy_buf = vec![0u8; DUMMY_PERIPHERAL_MAX_MSG_SIZE_BYTES as usize];
                let dummy_bytes_read_res = self.peripheral_interface.read(&mut dummy_buf);
                match dummy_bytes_read_res {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            self.handle_msg_from_peripheral(dummy_buf.clone());
                        }
                    }
                    Err(e) => {
                        println!("error reading from dummy subsystem: {:?}", e);
                    }
                }
            }
        }
    }
}

fn main() {
    /*
     * Setup interface for talking with the subsystems external peripheral
     * - this is a hardware device in most cases (though simulated in early development - using tcp)
     * - or this is a software or 'virtual' component in the obc and this handler interfaces with it via ipc
     */
    let dummy_subsystem_interface =
        TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_DUMMY_PORT).unwrap();

    /*
     * Setup interface for talking with other FSW components via IPC
     */
    let ipc_cmd_msg_dispatcher = IpcClient::new("dummy_handler".to_string()).unwrap();

    let mut dummy_handler = DummyHandler::new(dummy_subsystem_interface, ipc_cmd_msg_dispatcher);

    dummy_handler.run();
}
