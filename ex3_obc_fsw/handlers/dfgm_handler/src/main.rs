/*
Written By Devin Headrick and Rowan Rasmusson
Summer 2024

DFGM is a simple subsystem that only outputs a ~1250 byte packet at 1Hz, with no interface or control from the FSW.
The handler either chooses to collect the data or not depending on a toggle_data_collection flag.


TODO - If connection is lost with an interface, attempt to reconnect every 5 seconds
TOOD - Figure out way to use polymorphism and have the interfaces be configurable at runtime (i.e. TCP, UART, etc.)
TODO - Get state variables from a state manager (channels?) upon instantiation and update them as needed.
TODO - Setup a way to handle opcodes from messages passed to the handler

*/

use common::component_ids::DFGM;
use common::component_ids::GS;
use ipc::ipc_write;
use ipc::poll_ipc_clients;
use ipc::IpcClient;
use ipc::IPC_BUFFER_SIZE;
use tcp_interface::BUFFER_SIZE;
use tcp_interface::*;
use message_structure::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use common::{ports, opcodes}; 
use message_structure::*;


const DFGM_DATA_DIR_PATH: &str = "ex3_obc_fsw/handlers/dfgm_handler/dfgm_data";
const DFGM_PACKET_SIZE: usize = 1252;
const DFGM_INTERFACE_BUFFER_SIZE: usize = DFGM_PACKET_SIZE;

/// Opcodes for messages relating to DFGM functionality
// pub enum OpCode {
//     ToggleDataCollection, // toggles a flag which either enables or disables data collection from the DFGM
// }

/// Interfaces are option types incase they are not properly created upon running this handler, so the program does not panic
struct DFGMHandler {
    toggle_data_collection: bool,
    peripheral_interface: Option<TcpInterface>, // For communication with the DFGM peripheral [external to OBC]. Will be dynamic 
    msg_dispatcher_interface: Option<IpcClient>, // For communcation with other FSW components [internal to OBC] (i.e. message dispatcher)
    bulk_msg_dispatcher_interface: Option<IpcClient>
}

impl DFGMHandler {
    pub fn new(
        dfgm_interface: Result<TcpInterface, std::io::Error>,
        msg_dispatcher_interface: Result<IpcClient, std::io::Error>,
        bulk_msg_dispatcher_interface: Result<IpcClient, std::io::Error>
    ) -> DFGMHandler {
        //if either interfaces are error, print this
        if dfgm_interface.is_err() {
            println!(
                "Error creating DFGM interface: {:?}",
                dfgm_interface.as_ref().err().unwrap()
            );
        }
        if msg_dispatcher_interface.is_err() {
            println!(
                "Error creating dispatcher interface: {:?}",
                msg_dispatcher_interface.as_ref().err().unwrap()
            );
        }
        if bulk_msg_dispatcher_interface.is_err() {
            println!(
                "Error creating dispatcher interface: {:?}",
                bulk_msg_dispatcher_interface.as_ref().err().unwrap()
            );
        }

        DFGMHandler {
            toggle_data_collection: false,
            peripheral_interface: dfgm_interface.ok(),
            msg_dispatcher_interface: msg_dispatcher_interface.ok(),
            bulk_msg_dispatcher_interface: bulk_msg_dispatcher_interface.ok(),
        }
    }

    fn handle_msg_for_dfgm(&mut self, msg: Msg) -> Result<(), Error> {
        match msg.header.op_code {
            opcodes::dfgm::TOGGLE_DATA_COLLECTION => {
                if msg.msg_body[0] == 0 {
                    self.toggle_data_collection = false;
                    println!("Data toggle set to {}", self.toggle_data_collection);
                    Ok(())
                } else if msg.msg_body[0] == 1 {
                    self.toggle_data_collection = true;
                    println!("Data toggle set to {}", self.toggle_data_collection);
                    Ok(())
                } else {
                    eprintln!("Error: invalid msg body for opcode 0");
                    Err(Error::new(ErrorKind::InvalidData, "Invalid msg body for opcode 0 on DFGM"))
                }
            }
            opcodes::dfgm::GET_DFGM_DATA => {
                let data_to_send: Vec<u8> = "../handlers/dfgm_handler/dfgm_data".as_bytes().to_vec();
                let data_msg: Msg = Msg::new(0,0,GS,DFGM,0,data_to_send);
                let serialized_data_msg: Vec<u8> = serialize_msg(&data_msg)?;
                
                ipc_write(self.bulk_msg_dispatcher_interface.as_ref().unwrap().fd, &serialized_data_msg)?;
                println!("Sent path!");
                Ok(())
            }
            _ => {
                eprintln!("Error: invalid msg body for opcode 0");
                Err(Error::new(ErrorKind::NotFound, format!("Opcode {} not found for DFGM", msg.header.op_code)))
            }
        }
    }
    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) -> std::io::Result<()> {
        // Read and poll for input for a message
        loop {
            // Borrowing the dispatcher interfaces
            let msg_dispatcher_interface = self.msg_dispatcher_interface.as_mut().expect("Cmd_Msg_Disp has value of None");
            let bulk_msg_dispatcher_interface = self.bulk_msg_dispatcher_interface.as_mut().expect("Bulk_Msg_Disp has value None");

            let mut clients = vec![
                msg_dispatcher_interface,
                bulk_msg_dispatcher_interface,
            ];
            poll_ipc_clients(&mut clients)?;
            
            // Handling the bulk message dispatcher interface
            if let Some(cmd_msg_dispatcher) = self.msg_dispatcher_interface.as_mut() {
                if cmd_msg_dispatcher.buffer != [0u8; IPC_BUFFER_SIZE] {
                    let recv_msg: Msg = deserialize_msg(&cmd_msg_dispatcher.buffer).unwrap();
                    println!("Received and deserialized msg");
                    self.handle_msg_for_dfgm(recv_msg)?;
                    // TODO - clear the buffer!!
                }
            }
        
            if self.toggle_data_collection == true {
                let mut tcp_buf = [0u8;BUFFER_SIZE];
                let status = TcpInterface::read(&mut self.peripheral_interface.as_mut().unwrap(), &mut tcp_buf);
                match status {
                    Ok(data_len) => {
                        println!("Got data {:?}", tcp_buf);
                        store_dfgm_data(&tcp_buf)?;
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }
                //TODO - Convert bytestream into message struct
                //TODO - After receiving the message, send a response back to the dispatcher
                //TODO - handle the message based on its opcode
}


/// Write DFGM data to a file (for now --- this may changer later if we use a db or other storage)
/// Later on we likely want to specify a path to specific storage medium (sd card 1 or 2)
/// We may also want to implement something generic to handle 'payload data' storage so we can have it duplicated, stored in multiple locations, or compressed etc.
fn store_dfgm_data(data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(DFGM_DATA_DIR_PATH)?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/data", DFGM_DATA_DIR_PATH))?;
    file.write_all(data)?;
    Ok(())
}

fn main() -> Result<(), Error> {
    println!("Beginning DFGM Handler...");
    //For now interfaces are created and if their associated ports are not open, they will be ignored rather than causing the program to panic

    //Create TCP interface for DFGM handler to talk to simulated DFGM
    let dfgm_interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_DFGM_PORT);

    //Create Unix domain socket interface for DFGM handler to talk to command message dispatcher
    let msg_dispatcher_interface = IpcClient::new("dfgm_handler".to_string());

    // Create Unix domain socket for communication between DFGM handler and bulk message dispatcher
    let bulk_dispatcher_interface = IpcClient::new("dfgm_bulk".to_string());

    //Create DFGM handler
    let mut dfgm_handler = DFGMHandler::new(dfgm_interface, msg_dispatcher_interface, bulk_dispatcher_interface);

    dfgm_handler.run()
}