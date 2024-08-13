/*
written by devin headrick
summer 2024

todo - setup handler to re-attempt connection to subsystem if it fails or connection drops

*/

use common::opcodes;
use common::ports;
use message_structure::{deserialize_msg, AckMsg, CmdMsg, Msg, SerializeAndDeserialize};

use ipc::{poll_ipc_clients, IpcClient, IPC_BUFFER_SIZE};
use tcp_interface::{Interface, TcpInterface};

const dummy_max_msg_size_bytes: u8 = 128; //largest size of a data packet from dummy subsystem sim

struct DummyHandler {
    peripheral_interface: TcpInterface,
    dispatcher_interface: IpcClient,
}

impl DummyHandler {
    fn new(peripheral_interface: TcpInterface, dispatcher_interface: IpcClient) -> Self {
        Self {
            peripheral_interface,
            dispatcher_interface,
        }
    }

    // ----------------------------------------------------------------------------------------------------------------------
    /*
     * here goes functions related uniquely to the subsystem the handler is associated with (i.e. decryption for gs handler, )
     */
    fn set_dummy_subsystem_variable(&mut self, msg_body: Vec<u8>) {
        println!("set dummy subsystem variable called");
        //get first byte of msg_body as the variable to set
        let variable_value = msg_body[0] + 48;
        let set_dummy_var_cmd = "SET_DUMMY_VAR:";
        let outgoing_data = [set_dummy_var_cmd.as_bytes(), &variable_value.to_be_bytes()].concat();
        println!("sending message to dummy subsystem: {:?}", outgoing_data);
        let send_res = self.peripheral_interface.send(outgoing_data.as_slice());
    }
    fn get_dummy_subsystem_variable(&mut self) {
        println!("get dummy subsystem variable called");
        //todo - implement this with a dummy subsystem sim
    }

    // ----------------------------------------------------------------------------------------------------------------------
    /*
     * here goes functions for handling messages read from the subsystem peripheral (external device)
     * typically these parse the message, and use a match case on the opcode or other message fields determine what to do (what above fxns to call)
     */
    fn handle_dummy_msg_in(&mut self, dummy_msg: Vec<u8>) {
        // this is where we convert the subsystem messages into a meaningful format for the rest of the fsw and for operators to understand
        // this is all implementation specific - depends on the subsystems - what data looks like and how to handle it is in their user manual / docs
        //  - this is where the short fat implementation of code tightly coupled with the subsystem goes
        println!("received message from dummy subsystem: {:?}", dummy_msg);
    }
    // here goes 'handle' functions which are called upon receiving a message from an ipc interface - they are unique to the particular interface they are associated with
    fn handle_command_msg_in(&mut self, msg: Msg) {
        // parse the incoming message - use the 'from<u8>' trait implemented for the subsystems associated opcode enum
        let opcode = opcodes::DUMMY::from(msg.header.op_code);

        // call the appropriate function to handle the command
        match opcode {
            opcodes::DUMMY::SetDummyVariable => self.set_dummy_subsystem_variable(msg.msg_body),
            opcodes::DUMMY::GetDummyVariable => self.get_dummy_subsystem_variable(),
            // _ => println!("invalid opcode received"),
        }
    }

    /// Main loop for running the handler - this is where we listen (read) incomming messages from the subsystem perihpheral and ipc interfaces
    pub fn run(&mut self) {
        loop {
            {
                //TODO - Make the handler struct take a vector of interfaces to poll for messages from
                // as our design grows and handlers talk to more processes - this vec will grow to include other interfaces
                let mut ipc_client_vec = vec![&mut self.dispatcher_interface];
                poll_ipc_clients(&mut ipc_client_vec).unwrap();

                let dispatcher = &mut self.dispatcher_interface;
                // check if any of the ipc interfaces have received a message after polling them all
                if dispatcher.buffer != [0u8; IPC_BUFFER_SIZE] {
                    println!(
                        "received message from ipc interface: {:?}",
                        dispatcher.buffer
                    );

                    //------------------------------------------------------------------------------
                    // for now we know in command message uplink tall-thin its a command type message
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
                        self.handle_command_msg_in(msg);
                    }
                }
            }
            {
                let mut dummy_buf = vec![0u8; dummy_max_msg_size_bytes as usize];
                let dummy_bytes_read_res = self.peripheral_interface.read(&mut dummy_buf);
                match dummy_bytes_read_res {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            self.handle_dummy_msg_in(dummy_buf.clone());
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
    // setup interface for talking with the subsystem associated with this handler
    // - this is a hardware device in most cases (though simulated in early development - using tcp)
    // - or this is a software or 'virtual' component in the obc and this handler interfaces with it via ipc
    let dummy_subsystem_interface =
        TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_DUMMY_PORT).unwrap();

    // setup interfaces for communicating with other fsw components (typically ipc for communication between processes)
    let ipc_cmd_msg_dispatcher = IpcClient::new("dummy_handler".to_string()).unwrap();

    let mut dummy_handler = DummyHandler::new(dummy_subsystem_interface, ipc_cmd_msg_dispatcher);

    dummy_handler.run();
}
