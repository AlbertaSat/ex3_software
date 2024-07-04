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
use std::env;
use std::process;

const BUFFER_SIZE: usize = 1024;
use ipc_interface::read_socket;
use ipc_interface::IPCInterface;
use tcp_interface::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use common::ports; 

const DFGM_DATA_DIR_PATH: &str = "dfgm_data";
const DFGM_PACKET_SIZE: usize = 1252;
const DFGM_INTERFACE_BUFFER_SIZE: usize = DFGM_PACKET_SIZE;

const DISPATCHER_INTERFACER_BUFFER_SIZE: usize = 512;

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
            toggle_data_collection: true,
            peripheral_interface: dfgm_interface.ok(),
            dispatcher_interface: dispatcher_interface.ok(),
        }
    }

    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) {
        let args: Vec<String> = env::args().collect();

        if args.len() != 2 {
            eprintln!("Usage: {} <tcp_port>", args[0]);
            process::exit(1);
        }

    
        let tcp_port: u16 = args[1].parse().unwrap_or_else(|_| {
            eprintln!("Invalid TCP port number");
            process::exit(1);
        });

        // ------------------ Peripheral Interface Setup ------------------
        let tcp_interface = TcpInterface::new_client("127.0.0.1".to_string(), tcp_port);
    
        println!("TCP client connected to port {}", tcp_port);

        // ------------------ Dispatcher Interface Setup ------------------

        let ipc_interface = IPCInterface::new("dfgm_handler".to_string());

        // Read and poll for input for a message
        let mut socket_buf = vec![0u8; DFGM_INTERFACE_BUFFER_SIZE];
        loop {
            let n = read_socket(ipc_interface.fd, &mut socket_buf).unwrap();
            if n > 0 {
                store_dfgm_data(&socket_buf);
            }
        }



                //TODO - Convert bytestream into message struct
                //TODO - After receiving the message, send a response back to the dispatcher
                //TODO - handle the message based on its opcode
    }
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
