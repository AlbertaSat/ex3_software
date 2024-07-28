/*
Written By Ben Fisher, extreme referencing from Devin Headrick's work
Summer 2024

IRIS is a subsystem that is responsible for imaging the surface of the planet at schedulable times. It can store
images onboard and the OBC can then fetch these images from the IRIS to send to the ground station. The IRIS subsystem
should be completely controlled via the FSW, it does not ouput data except when asked. The handler receives commands
from the OBC receiver who receives commands from the groundstation via UHF. It may be that certain commands from the OBC 
require multiple commands on the IRIS to activate, the handler should recognize this.


TODO - implement iris handler and interfacing (need to figure out how)

TODO - If connection is lost with an interface, attempt to reconnect every 5 seconds
TOOD - Figure out way to use polymorphism and have the interfaces be configurable at runtime (i.e. TCP, UART, etc.)
TODO - Get state variables from a state manager (channels?) upon instantiation and update them as needed.
TODO - Setup a way to handle opcodes from messages passed to the handler


*/

use common::{opcodes, ports};
use ipc_interface::read_socket;
use ipc_interface::IPCInterface;
use message_structure::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use tcp_interface::*;

const IRIS_DATA_DIR_PATH: &str = "iris_data";
const IRIS_PACKET_SIZE: usize = 1252;
const IRIS_INTERFACE_BUFFER_SIZE: usize = IRIS_PACKET_SIZE;

/// Opcodes for messages relating to IRIS functionality
// pub enum OpCode {

// }

/// Interfaces are option types incase they are not properly created upon running this handler, so the program does not panic
struct IRISHandler {
    toggle_sensor: bool,
    peripheral_interface: Option<TcpInterface>, // For communication with the IRIS peripheral [external to OBC]. Will be dynamic
    dispatcher_interface: Option<IPCInterface>, // For communcation with other FSW components [internal to OBC] (i.e. message dispatcher)
}

impl IRISHandler {
    pub fn new(
        iris_interface: Result<TcpInterface, std::io::Error>,
        dispatcher_interface: Result<IPCInterface, std::io::Error>,
    ) -> IRISHandler {
        //if either interfaces are error, print this
        if iris_interface.is_err() {
            println!(
                "Error creating IRIS interface: {:?}",
                iris_interface.as_ref().err().unwrap()
            );
        }
        if dispatcher_interface.is_err() {
            println!(
                "Error creating dispatcher interface: {:?}",
                dispatcher_interface.as_ref().err().unwrap()
            );
        }

        IRISHandler {
            toggle_sensor: false,
            peripheral_interface: iris_interface.ok(),
            dispatcher_interface: dispatcher_interface.ok(),
        }
    }

    fn handle_msg_for_iris(&mut self, msg: Msg){
        
        let (command_msg, success) = match msg.header.op_code {
            opcodes::iris::RESET=> {
                ("RST", true)
            }
            // Image commands
            opcodes::iris::TOGGLE_SENSOR=> {
                if msg.msg_body[0] == 1 {
                    ("ON", true)
                } else if msg.msg_body[0] == 0 {
                    ("OFF", true)
                } else {
                    ("Error: invalid msg body for opcode 1", false)
                }
            }
            opcodes::iris::CAPTURE_IMAGE=> {
                ("TKI", true)
            }
            opcodes::iris::FETCH_IMAGE=> {
                // Assumes that there are not more than 255 images being request at any one time
                (&*format!("FTI:{}", msg.msg_body[0]),true)
            }
            opcodes::iris::GET_IMAGE_SIZE=> {
                // Currently can only access the first 255 images stored on IRIS, will be updated if needed
                (&*format!("FSI:{}", msg.msg_body[0]),true)
            }
            opcodes::iris::GET_N_IMAGES_AVAILABLE=> {
                ("FNI", true)
            }
            opcodes::iris::DEL_IMAGE=> {
                (&*format!("DTI:{}", msg.msg_body[0]),true)
            }
            // Housekeeping commands
            opcodes::iris::GET_TIME=> {
                ("FTT", true)
            }
            opcodes::iris::SET_TIME=> {
                // Placeholder for reading the total time need to determine how we will handle >255 values (ie. epoch time)
                (&*format!("STT:{}", msg.msg_body[0]),true)
            }
            opcodes::iris::GET_HK=> {
                ("FTH", true)
            }
            _ => {
                (&*format!("Opcode {} not found for IRIS", msg.header.op_code), false)
                
            }
        };
        if success {
            let status = TcpInterface::send(&mut self.peripheral_interface.as_mut().unwrap(), command_msg.as_bytes());
            if write_status(status, command_msg) {
                // Read response from the subsystem
                let mut tcp_buf = [0u8; BUFFER_SIZE];
                let status = TcpInterface::read(&mut self.peripheral_interface.as_mut().unwrap(), &mut tcp_buf);
                match status {
                    Ok(_data_len) => { println!("Got data {:?}", std::str::from_utf8(&tcp_buf)); }
                    Err(e) => { println!("Error: {}", e); }
                }

            }
            return;
        }
        eprintln!("{}", command_msg);

    }
    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) -> std::io::Result<()> {
        // Read and poll for input for a message
        let mut socket_buf = vec![0u8; IRIS_INTERFACE_BUFFER_SIZE];
        loop {
            if let Ok(n) = read_socket(
                self.dispatcher_interface.clone().unwrap().fd,
                &mut socket_buf,
            ) {
                if n > 0 {
                    let recv_msg: Msg = deserialize_msg(&socket_buf).unwrap();
                    self.handle_msg_for_iris(recv_msg);

                    socket_buf.flush()?;
                }
            }
        }
    }

    //TODO - Convert bytestream into message struct
    //TODO - After receiving the message, send a response back to the dispatcher
    //TODO - handle the message based on its opcode
}

/// Write IRIS data to a file (for now --- this may changer later if we use a db or other storage)
/// Later on we likely want to specify a path to specific storage medium (sd card 1 or 2)
/// We may also want to implement something generic to handle 'payload data' storage so we can have it duplicated, stored in multiple locations, or compressed etc.
fn store_iris_data(data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(IRIS_DATA_DIR_PATH)?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/data", IRIS_DATA_DIR_PATH))?;
    file.write_all(data)?;
    Ok(())
}

fn write_status(status: Result<usize, Error>, cmd: &str) -> bool{
    match status {
        Ok(_data_len) => {
            println!("Command {} successfully sent", cmd);
            true
        }
        Err(e) => {
            println!("Error: {}", e);
            false
        }
    }
}

fn main() {
    println!("Beginning IRIS Handler...");
    //For now interfaces are created and if their associated ports are not open, they will be ignored rather than causing the program to panic

    //Create TCP interface for IRIS handler to talk to simulated IRIS
    let iris_interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_IRIS_PORT);

    //Create IPC interface for IRIS handler to talk to message dispatcher
    let dispatcher_interface = IPCInterface::new("iris_handler".to_string());

    //Create IRIS handler
    let mut iris_handler = IRISHandler::new(iris_interface, dispatcher_interface);

    iris_handler.run();
}
