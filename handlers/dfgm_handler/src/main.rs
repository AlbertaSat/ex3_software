/*
Written By Devin Headrick
Summer 2024

DFGM is a simple subsystem that only outputs a ~1250 byte packet at 1Hz, with no interface or control from the FSW.
The handler either chooses to collect the data or not depending on a toggle_data_collection flag.


TODO - If connection is lost with an interface, attempt to reconnect every 5 seconds
TOOD - Figure out way to use polymorphism and have the interfaces be configurable at runtime (i.e. TCP, UART, etc.)
TODO - Get state variables from a state manager (channels?) upon instantiation and update them as needed.
TODO - Setup a way to handle opcodes from messages passed to the handler

*/

use ipc_interface::read_socket;
use ipc_interface::IPCInterface;
use tcp_interface::TCP_BUFFER_SIZE;
use tcp_interface::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use common::ports; 
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
    peripheral_interface: Option<TcpInterface>, // For communication with the DFGM peripheral [external to OBC]
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

    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) -> std::io::Result<()> {

        // ------------------ Peripheral Interface Setup ------------------
    
        println!("TCP client connected to port {}", ports::SIM_DFGM_PORT);

        // ------------------ Dispatcher Interface Setup ------------------

        // Read and poll for input for a message
        let mut socket_buf = vec![0u8; DFGM_INTERFACE_BUFFER_SIZE];
        loop {
            if let Ok(n) = read_socket(self.dispatcher_interface.clone().unwrap().fd, &mut socket_buf) {
                if n > 0 {
                    let recv_msg: Msg = deserialize_msg(&socket_buf).unwrap();
                    if recv_msg.header.op_code == 0 && self.toggle_data_collection == false {
                        self.toggle_data_collection = true;
                    } else if recv_msg.header.op_code == 0 && self.toggle_data_collection == true {
                        self.toggle_data_collection = false;
                    }
                    
                    println!("Data toggle set to {}", self.toggle_data_collection);
                    socket_buf.flush()?;
                }
            }
            if self.toggle_data_collection == true {
                let mut tcp_buf = [0u8;TCP_BUFFER_SIZE];
                let status = TcpInterface::read(&mut self.peripheral_interface.as_mut().unwrap(), &mut tcp_buf);
                match status {
                    Ok(data_len) => {
                        println!("Got data {:?}", tcp_buf);
                        store_dfgm_data(&tcp_buf);
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

fn main() {
    println!("Beginning DFGM Handler...");
    //For now interfaces are created and if their associated ports are not open, they will be ignored rather than causing the program to panic

    //Create TCP interface for DFGM handler to talk to simulated DFGM
    let dfgm_interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_DFGM_PORT);

    //Create TCP interface for DFGM handler to talk to message dispatcher
    let dispatcher_interface = Ok(IPCInterface::new("dfgm_handler".to_string()));

    //Create DFGM handler
    let mut dfgm_handler = DFGMHandler::new(dfgm_interface, dispatcher_interface);

    dfgm_handler.run();
}
