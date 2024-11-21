/*
Written by _
Fall 2024

Three things to do:
    1. handle opcodes using enums and match
    2. three interfaces: msg_dispatcher (alr exists, where it gets op codes/msgs form fground station), 
    one in the example, talks to the sim gps rn, and get time lat long etc 
    and the one to send thingfs backt o the ground station (example of that in the shell handler file rn ie when send things back to client, 


*/

use log::{debug, trace, warn};
use common::logging::*;
use std::io::Error;

use ipc::{IpcClient, IpcServer, IPC_BUFFER_SIZE, ipc_write, poll_ipc_clients, poll_ipc_server_sockets};
use common::message_structure::*;

use std::{thread, time};

struct GPSHandler {
    // Olivia and ben write the interface from the example here!!!!
    msg_dispatcher_interface: Option<IpcServer>, // For communcation with other FSW components [internal to OBC]
}

impl GPSHandler {
    pub fn new(
        msg_dispatcher_interface: Result<IpcServer, std::io::Error>,
    ) -> GPSHandler {
        if msg_dispatcher_interface.is_err() {
            warn!(
                "Error creating dispatcher interface: {:?}",
                msg_dispatcher_interface.as_ref().err().unwrap()
            );
        }
        GPSHandler {
            msg_dispatcher_interface: msg_dispatcher_interface.ok(),
        }
    }

    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) -> std::io::Result<()> {
        // Poll for messages
        loop {
            // First, take the Option<IpcClient> out of `self.dispatcher_interface`
            // This consumes the Option, so you can work with the owned IpcClient
            let msg_dispatcher_interface = self.msg_dispatcher_interface.take().expect("Cmd_Disp has value of None");

            // Create a mutable Option<IpcClient> so its lifetime persists
            let mut msg_dispatcher_interface_option = Some(msg_dispatcher_interface);

            // Now you can borrow this mutable option and place it in the vector
            let mut server: Vec<&mut Option<IpcServer>> = vec![
                &mut msg_dispatcher_interface_option,
            ];

            poll_ipc_server_sockets(&mut server);

            // restore the value back into `self.dispatcher_interface` after polling. May have been mutated
            self.msg_dispatcher_interface = msg_dispatcher_interface_option;

            // Handling the bulk message dispatcher interface
            let msg_dispatcher_interface = self.msg_dispatcher_interface.as_ref().unwrap();
            if msg_dispatcher_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
                let recv_msg: Msg = deserialize_msg(&msg_dispatcher_interface.buffer).unwrap();
                debug!("Received and deserialized msg");
                self.handle_msg(recv_msg)?;
            }
        }
    }
// HANDLE MATCH STATEMENTS
    fn handle_msg(&mut self, msg: Msg) -> Result<(), Error> {
        self.msg_dispatcher_interface.as_mut().unwrap().clear_buffer();
        println!("GPS msg opcode: {} {:?}", msg.header.op_code, msg.msg_body);
        // handle opcodes: https://docs.google.com/spreadsheets/d/1rWde3jjrgyzO2fsg2rrVAKxkPa2hy-DDaqlfQTDaNxg/edit?gid=0#gid=0
        Ok(())
    }
}

fn main() {
    let log_path = "ex3_obc_fsw/handlers/gps_handler/logs";
    init_logger(log_path);

    trace!("Starting GPS Handler...");

    // Create Unix domain socket interface for to talk to message dispatcher
    let msg_dispatcher_interface = IpcServer::new("GPS".to_string());

    let mut gps_handler = GPSHandler::new(msg_dispatcher_interface);

    // example (TODO add gps_interface to GPSHandler object and poll in run loop)
    let mut gps_interface = IpcClient::new("gps_device".to_string()).ok();      // connect("/tmp/fifo_socket_gps_device")
    let _ = ipc_write(&gps_interface.as_ref().unwrap().fd, "time".as_bytes());  // send("time")
    thread::sleep(time::Duration::from_millis(100));                            // wait (only for example)
    let _ = poll_ipc_clients(&mut vec![&mut gps_interface]);                    // recv()
    println!("Got \"{}\"", String::from_utf8(gps_interface.as_mut().unwrap().read_buffer()).unwrap());

    let _ = gps_handler.run();
}
