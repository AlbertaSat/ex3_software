/*
Written By Devin Headrick
Summer 2024

Handler will
    - handle all communication with their respective subsystem via interface
    - communicate with other FSW via IPC
    - take opcodes from messages passed to them, and execute their related functionality


DFGM is a simple subsystem that only outputs a ~1250 byte packet at 1Hz, with no interface or control from the FSW.
The handler either chooses to collect the data or not.


TODO - If connection is lost with an interface, attempt to reconnect every 5 seconds

TOOD - Figure out way to use polymorphism and have the interfaces be configurable at runtime (i.e. TCP, UART, etc.)
TODO - get state variables from a state manager (channels?) upon instantiation and update them as needed.

*/

use interfaces::*;
use std::borrow::Borrow;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

const DFGM_SIM_PORT: u16 = 1802;
const DISPATCHER_PORT: u16 = 1900;
const DFGM_DATA_DIR_PATH: &str = "dfgm_data";

/// Opcodes for messages relating to DFGM functionality
pub enum OpCode {
    ToggleDataCollection, // toggles a flag which either enables or disables data collection from the DFGM
}

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

    pub fn run(&mut self) {
        // call async read and write on interfaces
        // handle incomming messages when they are recevied and send outgoing messages

        // NOTE: A seperate channel PAIR is needed for both reading and writing to an interface.
        // one pair handles communication between the async read and the handler, and the other pair handles communication between the handler and the async write

        //Setup channel pairs for interface threads to communicate with handler
        let (dfgm_tx, dfgm_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
        //let (dispatcher_tx, dispatcher_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

        if let Some(interface) = self.peripheral_interface.clone() {
            async_read(interface, dfgm_tx.clone());
        }

        // if let Some(interface) = self.dispatcher_interface.clone() {
        //     async_read(interface, dispatcher_tx.clone());
        // }

        //dispatcher_tx.send(b"Hello from DFGM Handler! \n \n \n".to_vec());

        // //setup a seperate channel pair for writing to the dispatcher
        // Async write opens a thread with the recevier, and we write to it by writing data to the sender
        let (dispatcher_tx_write, dispatcher_rx_write): (Sender<Vec<u8>>, Receiver<Vec<u8>>) =
            mpsc::channel();


        dispatcher_tx_write
            .send(b"Hello from DFGM Handler! \n".to_vec())
            .expect("Failed to send message to dispatcher");
        
        std::thread::sleep(std::time::Duration::from_secs(1));

        if let Some(interface) = self.dispatcher_interface.clone() {
            async_write(interface, dispatcher_rx_write);
        }
        loop {
            // if let Ok(data) = dfgm_rx.try_recv() {
            //     if data.is_empty() {
            //         continue;
            //     }
            //     println!("Received DFGM Data{:?}", data);
            //     if self.toggle_data_collection {
            //         store_dfgm_data(&data);
            //     }
            // }

            // if let Ok(data) = dispatcher_rx.try_recv() {
            //     if data.is_empty() {
            //         continue;
            //     }
            //     println!("Received Dispatcher Data{:?}", data);
            //     //TODO - Convert bytestream into message struct
            //     //TODO - handle the message based on its opcode
            // }
           
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

    //For now interfaces are created and if their associated ports are not open, they will be ignored

    //Create TCP interface for DFGM handler to talk to simulated DFGM
    let dfgm_interface = TcpInterface::new("127.0.0.1".to_string(), DFGM_SIM_PORT);

    //Create TCP interface for DFGM handler to talk to message dispatcher
    let dispatcher_interface = TcpInterface::new("127.0.0.1".to_string(), DISPATCHER_PORT);

    //Create DFGM handler
    let mut dfgm_handler = DFGMHandler::new(dfgm_interface, dispatcher_interface);

    dfgm_handler.run();
}
