/*
Written by Devin Headrick
Summer 2024

TODO - Setup handler to re-attempt connection to subsystem if it fails or connection drops

*/

use common::opcodes;
use common::ports;
use message_structure::{AckMsg, CmdMsg, SerializeAndDeserialize};

use ipc::{poll_ipc_clients, IpcClient, IPC_BUFFER_SIZE};
use tcp_interface::{Interface, TcpInterface};

const DUMMY_MAX_MSG_SIZE_BYTES: u8 = 128;

/*
 * Here goes functions related uniquely to the subsystem the handler is associated with (i.e. decryption for GS handler, )
*/
// ----------------------------------------------------------------------------------------------------------------------
fn set_dummy_subsystem_variable() {
    // - write to the interface for the subsystem (tcp for sims)
}
fn get_dummy_subsystem_variable() {
    // - write to and then read from the interface for the subsystem (tcp for sims)
}

/*
 * Here goes functions for handling messages read.
 * Typically these parse the message, and use a match case on the opcode or other message fields determine what to do (what above fxns to call)
*/
// ----------------------------------------------------------------------------------------------------------------------
/// Handle a message received from the subsystem associated with this handler
fn handle_dummy_msg_in(dummy_msg: Vec<u8>) {
    // This is where we convert the subsystem messages into a meaningful format for the rest of the FSW and for operators to understand

    // THIS IS ALL IMPLEMENTATION SPECIFIC - DEPENDS ON THE SUBSYSTEMS - WHAT DATA LOOKS LIKE AND HOW TO HANDLE IT IS IN THEIR USER MANUAL / DOCS
    //  - this is where the short fat implementation of code tightly coupled with the subsystem goes
    println!("Received message from dummy subsystem: {:?}", dummy_msg);
}
// Here goes 'handle' functions which are called upon receiving a message from an IPC interface - they are unique to the particular interface they are associated with
fn handle_command_msg_in(msg: CmdMsg) {
    // Parse the incoming message - use the 'From<u8>' trait implemented for the subsystems associated opcode enum
    let opcode = opcodes::DUMMY::from(msg.opcode);

    // Call the appropriate function to handle the command
    match opcode {
        opcodes::DUMMY::SetDummyVariable => set_dummy_subsystem_variable(),
        opcodes::DUMMY::GetDummyVariable => get_dummy_subsystem_variable(),
        // _ => println!("Invalid opcode received"),
    }
}

fn main() {
    // Setup interface for talking with the subsystem associated with this handler
    // - this is a hardware device in most cases (though simulated in early development - using TCP)
    // - or this is a software or 'virtual' component in the OBC and this handler interfaces with it via IPC
    let mut dummy_subsystem_interface =
        TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_DUMMY_PORT).unwrap();
    let mut dummy_buf = vec![0u8; DUMMY_MAX_MSG_SIZE_BYTES as usize];

    // Setup interfaces for communicating with other FSW components (typically IPC for communication between processes)
    let mut ipc_cmd_msg_dispatcher = IpcClient::new("cmd_msg_dummy_handler".to_string()).unwrap();
    // Setup vector of ipc clients for polling all for input data
    let mut ipc_client_vec = vec![&mut ipc_cmd_msg_dispatcher];

    //Enter main loop here to poll for incoming messages from previously setup interfaces
    loop {
        poll_ipc_clients(&mut ipc_client_vec).unwrap();

        // Check if any of the ipc interfaces have received a message after polling them all
        for ipc_client in ipc_client_vec.iter_mut() {
            if ipc_client.buffer != [0u8; IPC_BUFFER_SIZE] {
                println!(
                    "Received message from ipc interface: {:?}",
                    ipc_client.buffer
                );
                // In this loop we don't know which ipc socket this 'client' object is associated with
                // Originally the message type was known because the ipc socket is only setup to pass a particular type of message
                // - if we later want to pass multiple types of messages over the same interface, we will need to include a 'type' field in the message
                // THIS REQUIRES DESIGN DECISION
                // BUT HOW DO YOU DETERMINE THE 'TYPE' OF MESSAGE TO KNOW HOW TO DESERIALIZE IT

                //------------------------------------------------------------------------------
                // for now we know in command message uplink tall-thin its a command type message
                let deserialized_msg_res =
                    CmdMsg::deserialize_from_bytes(ipc_client.buffer.to_vec());
                match deserialized_msg_res {
                    Ok(deserialized_msg) => {
                        handle_command_msg_in(deserialized_msg);
                    }
                    Err(e) => {
                        println!("Error deserializing message: {:?}", e);
                        //TODO - send ack message back to msg source that message deserialization failed
                        // - (include error code, and where this failure occured i.e. the dummy_handler)
                    }
                }

                ipc_client.clear_buffer();
            }
        }

        let dummy_bytes_read_res = dummy_subsystem_interface.read(&mut dummy_buf);
        match dummy_bytes_read_res {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    handle_dummy_msg_in(dummy_buf.clone());
                }
            }
            Err(e) => {
                println!("Error reading from dummy subsystem: {:?}", e);
            }
        }
    }
}
