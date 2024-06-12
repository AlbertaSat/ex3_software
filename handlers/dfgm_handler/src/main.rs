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

use interfaces::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

const DFGM_SIM_PORT: u16 = 1802;
const DISPATCHER_PORT: u16 = 1900;
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
    dispatcher_interface: Option<TcpInterface>, // For communcation with other FSW components [internal to OBC] (i.e. message dispatcher)
}

impl DFGMHandler {
    pub fn new(
        dfgm_interface: Result<TcpInterface, std::io::Error>,
        dispatcher_interface: Result<TcpInterface, std::io::Error>,
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
        // NOTE: A seperate channel PAIR each is needed for the main handler process to communicate with a thread.
        // one pair handles communication between the async_read and the handler, and the other pair handles communication between the handler and the async_write

        // ------------------ Peripheral Interface Setup ------------------
        let (dfgm_reader_tx_ch, dfgm_reader_rx_ch): (Sender<Vec<u8>>, Receiver<Vec<u8>>) =
            mpsc::channel();

        if let Some(interface) = self.peripheral_interface.clone() {
            async_read(
                interface,
                dfgm_reader_tx_ch.clone(),
                DFGM_INTERFACE_BUFFER_SIZE,
            );
        }
        // NOTE: DFGM doesnt setup a 'dfgm_writer' thread because it only reads data from the DFGM

        // ------------------ Dispatcher Interface Setup ------------------
        let (disp_writer_tx_ch, disp_writer_rx_ch): (Sender<Vec<u8>>, Receiver<Vec<u8>>) =
            mpsc::channel();

        if let Some(interface) = self.dispatcher_interface.clone() {
            async_write(interface, disp_writer_rx_ch);
        }

        let (disp_reader_tx_ch, disp_reader_rx_ch): (Sender<Vec<u8>>, Receiver<Vec<u8>>) =
            mpsc::channel();

        if let Some(interface) = self.dispatcher_interface.clone() {
            async_read(
                interface,
                disp_reader_tx_ch.clone(),
                DISPATCHER_INTERFACER_BUFFER_SIZE,
            );
        }

        // Test sending dummy message
        disp_writer_tx_ch
            .send(b"Hello from DFGM Handler! \n".to_vec())
            .expect("Failed to send message to dispatcher");

        std::thread::sleep(std::time::Duration::from_secs(1)); //Sleep to let the message be sent

        loop {
            if let Ok(data) = dfgm_reader_rx_ch.try_recv() {
                if data.is_empty() {
                    continue;
                }
                println!("Received DFGM Data{:?}", data);
                if self.toggle_data_collection {
                    store_dfgm_data(&data);
                }
            }

            if let Ok(data) = disp_reader_rx_ch.try_recv() {
                if data.is_empty() {
                    continue;
                }
                println!("Received Dispatcher Data{:?}", data);
                //TODO - Convert bytestream into message struct
                //TODO - After receiving the message, send a response back to the dispatcher
                //TODO - handle the message based on its opcode
            }
        }
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
    let dfgm_interface = TcpInterface::new_client("127.0.0.1".to_string(), DFGM_SIM_PORT);

    //Create TCP interface for DFGM handler to talk to message dispatcher
    let dispatcher_interface = TcpInterface::new_client("127.0.0.1".to_string(), DISPATCHER_PORT);

    //Create DFGM handler
    let mut dfgm_handler = DFGMHandler::new(dfgm_interface, dispatcher_interface);

    dfgm_handler.run();
}
