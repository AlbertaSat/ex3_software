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

    fn handle_msg_for_iris(&mut self, msg: Msg) -> Result<(), Error> {
        match msg.header.op_code {
            opcodes::iris::TOGGLE_SENSOR=> {
                if msg.msg_body[0] == 0 {
                    self.toggle_sensor = false;
                    Ok(())
                } else if msg.msg_body[0] == 1 {
                    self.toggle_sensor = true;
                    Ok(())
                } else {
                    eprintln!("Error: invalid msg body for opcode 0");
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid msg body for opcode 0 on IRIS",
                    ))
                }
            }
            _ => {
                eprintln!("Error: invalid msg body for opcode 0");
                Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Opcode {} not found for IRIS", msg.header.op_code),
                ))
            }
        }
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
                    self.handle_msg_for_iris(recv_msg)?;
                    println!("Data toggle set to {}", self.toggle_sensor);
                    socket_buf.flush()?;
                }
            }
            if self.toggle_sensor == true {
                // TODO: Swap out dfgm code for IRIS code
                let mut tcp_buf = [0u8; BUFFER_SIZE];
                let status = TcpInterface::read(
                    &mut self.peripheral_interface.as_mut().unwrap(),
                    &mut tcp_buf,
                );
                match status {
                    Ok(data_len) => {
                        println!("Got data {:?}", tcp_buf);
                        store_iris_data(&tcp_buf)?;
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

fn main() {
    println!("Beginning IRIS Handler...");
    //For now interfaces are created and if their associated ports are not open, they will be ignored rather than causing the program to panic

    //Create TCP interface for IRIS handler to talk to simulated IRIS
    let iris_interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_IRIS_PORT);

    //Create TCP interface for IRIS handler to talk to message dispatcher
    let dispatcher_interface = IPCInterface::new("iris_handler".to_string());

    //Create IRIS handler
    let mut iris_handler = IRISHandler::new(iris_interface, dispatcher_interface);

    iris_handler.run();
}
