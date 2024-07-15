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
use ipc_interface::read_socket;
use ipc_interface::send_over_socket;
use ipc_interface::IPCInterface;
use tcp_interface::BUFFER_SIZE;
use tcp_interface::*;
use message_structure::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use common::{ports, opcodes}; 
use message_structure::*;


const DFGM_DATA_DIR_PATH: &str = "dfgm_data";
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
    dispatcher_interface: Option<IPCInterface>, // For communcation with other FSW components [internal to OBC] (i.e. message dispatcher)
}

impl DFGMHandler {
    pub fn new(
        dfgm_interface: Result<TcpInterface, std::io::Error>,
        dispatcher_interface: Result<IPCInterface, std::io::Error>,
    ) -> DFGMHandler {
        //if either interfaces are error, print this
        if dfgm_interface.is_err() {
            println!(
                "Error creating DFGM interface: {:?}",
                dfgm_interface.as_ref().err().unwrap()
            );
        }
        if dispatcher_interface.is_err() {
            println!(
                "Error creating dispatcher interface: {:?}",
                dispatcher_interface.as_ref().err().unwrap()
            );
        }

        DFGMHandler {
            toggle_data_collection: false,
            peripheral_interface: dfgm_interface.ok(),
            dispatcher_interface: dispatcher_interface.ok(),
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
                let data_to_send: Vec<u8> = get_dfgm_data()?;
                let data_msg: Msg = Msg::new(0,0,GS,DFGM,0,data_to_send);
                let serialized_data_msg: Vec<u8> = serialize_msg(&data_msg)?;
                
                println!("sending msg {:?}", data_msg);
                println!("Length of data after serialization {} bytes", data_msg.msg_body.len() + 5);
                println!("Sending {:?}", serialized_data_msg);
                let n = send_over_socket(self.dispatcher_interface.as_ref().unwrap().fd, serialized_data_msg)?;
                println!("Sent data!");
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
        let mut socket_buf = vec![0u8; DFGM_INTERFACE_BUFFER_SIZE];
        loop {
            if let Ok(n) = read_socket(self.dispatcher_interface.clone().unwrap().fd, &mut socket_buf) {
                if n > 0 {
                    let recv_msg: Msg = deserialize_msg(&socket_buf).unwrap();
                    println!("Received msg");
                    self.handle_msg_for_dfgm(recv_msg)?;
                    socket_buf.flush()?;
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
/// This function will take all current data that is stored for the dfgm and
/// write it to the message dispatcher. There it will be handled accordingly
fn get_dfgm_data() -> Result<Vec<u8>, std::io::Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(format!("{}/data", DFGM_DATA_DIR_PATH))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn main() -> Result<(), Error> {
    println!("Beginning DFGM Handler...");
    //For now interfaces are created and if their associated ports are not open, they will be ignored rather than causing the program to panic

    //Create TCP interface for DFGM handler to talk to simulated DFGM
    let dfgm_interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_DFGM_PORT);

    //Create Unix domain socket interface for DFGM handler to talk to message dispatcher
    let dispatcher_interface = IPCInterface::new("dfgm_handler".to_string());

    //Create DFGM handler
    let mut dfgm_handler = DFGMHandler::new(dfgm_interface, dispatcher_interface);

    dfgm_handler.run()
}