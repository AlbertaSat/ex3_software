/*
Written by _
Fall 2024

Background info:
    COMMAND MSG UPLINK:
    - Command message (sent from GS thru comms, to the GPS) will contain a msg header and a msg body.
    - For the GPS, only the header matters there is no extra command data related to the opcode (as on Nov 18 2024)
    - Question: handle message error detection in the message body?
    BULK MSG DOWNLINK:
    - In the handler, make a new message with the path to bulk data
    - Send that path to the Bulk Msg Dispatcher!
    - The Bulk Msg Dipatcher slices data into smaller pieces, and then:
        - Send Cmd Msg to GS handler: "this is the size of buffer you need to prepare for!"
        - When GS allocates this buffer and sends an ACK back to Bulk Msg Dispatcher, 
        - then Bulk Msg Dispatcher sends Msg's in its vector of the IPC socket
        - GS will continuously read and store these!

*/

//GPS data is primarily only used for
// 1. TIME SYNCINg
// 2. COORDINATE TAGGING of IRIS PHOTOS

use log::{debug, trace, warn};
use logging::*;
use std::io::Error;

use ipc::{IpcClient, IpcServer, IPC_BUFFER_SIZE, ipc_write, poll_ipc_clients, poll_ipc_server_sockets};
use message_structure::*;

use std::{thread, time};

const SIM_GPS = "gps_device"

struct GPSHandler {
    // Olivia and ben write the interface from the example here!!!!
    msg_dispatcher_interface: Option<IpcServer>, // For communcation with other FSW components [internal to OBC]
    gs_interface: Option<IpcClient> // For sending messages to the GS through the coms_handler
    /* Comments from Hari:
    "Why should the GPS ever directly communicate to the ground station? No reason. 
    The gps isnt on the BUS. The gps isnt connected to the spacecraft bus, It HAS to go to the OBC
    Talk to OBC over PHYSICAL port.
    The GPS shouldnt decide where the data goes. ."
    */
    gps_interface: Option<IpcClient> // For sending messages to the GPS 
}

impl GPSHandler {
    // this is an implementation block for the struct GPSHandler. 
    pub fn new(msg_dispatcher_interface: Result<IpcServer, std::io::Error>,) -> Self {
    //  creates a new GPSHandler object, setting up its internal message dispatcher interface for communication with the dispatcher.
    //  We should ideally have only one active GPSHandler instance at a time.
    //  Does not create/dispatch new messages--simply initializes listening
    // "new" is an associated function that returns a new instance of GPSHandler. "Self" is an alias for "GPSHandler".
    // use the enum Result<T,E> for error handling (see below the err and the ok)
        if msg_dispatcher_interface.is_err() {
            warn!(
                "Error creating dispatcher interface: {:?}",
                msg_dispatcher_interface.as_ref().err().unwrap()
            );
        }
        if gs_interface.is_err(){
            warn!(
                "Error creating gs interface: {:?}",
                gs_interface.as_ref().err().unwrap()
            );
        }
        if gps_interface.is_err(){
            warn!(
                "Error creating gps interface: {:?}",
                gps_interface.as_ref().err().unwrap()
            );
        }
        GPSHandler {
            msg_dispatcher_interface: msg_dispatcher_interface.ok(),
            gs_interface: gs_interface.ok(),
            gps_interface: gps_interface.ok(),
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

            // restore the value back into `self.msg_dispatcher_interface` after polling. May have been mutated
            self.msg_dispatcher_interface = msg_dispatcher_interface_option;

            // Handling the bulk message dispatcher interface
            let msg_dispatcher_interface = self.msg_dispatcher_interface.as_ref().unwrap();
            
            if msg_dispatcher_interface.buffer != [0u8; IPC_BUFFER_SIZE] { 
                // "0u8" is the smallest value repr by u8 type. 
                // "[0u8; IPC_BUFFER_SIZE]" means an array of IPC_BUFFER_SIZE filled with u8 zeroes.
                let recv_msg: Msg = deserialize_msg(&msg_dispatcher_interface.buffer).unwrap();
                debug!("Received and deserialized msg");
                self.handle_msg_for_gps(recv_msg)?;
            }
        }
    }

    fn handle_msg_for_gps(&mut self, msg: Msg) -> Result<(), Error> {
        //match the opcodes with the message header op_code
        //returns none if Ok, Error if err
        self.msg_dispatcher_interface.as_mut().unwrap().clear_buffer(); //Question: why this line?
        println!("GPS msg opcode: {} {:?}", msg.header.op_code, msg.msg_body);
        // handle opcodes: https://docs.google.com/spreadsheets/d/1rWde3jjrgyzO2fsg2rrVAKxkPa2hy-DDaqlfQTDaNxg/edit?gid=0#gid=0
        let (cmd_msg, success) = match opcodes::GPS::from(msg.header.op_code){
            //for now im using the simulated gps commands but this will change when we get the actual gps commands
            opcodes::GPS::GetLatLong => {   
                trace!("Getting latitude and longitude");
                ("latlong", true)
            }
            opcodes::GPS::GetUTCTime => {
                trace!("Getting UTC time");
                ("time", true)
            }
            opcodes::GPS::GetHK => {
                trace!("Getting HK");
                ("ping", true)
            }
            opcodes::GPS::Reset => {
                trace!("Resetting");
                //TODO: WE DONT HAVE RESET ON THE SIM GPS RN...
                ("NOT IMPLEMENTED YET", true)
            }
            _ => { //match case for everything else 
                warn!(
                    "{}",
                    format!("Error: Opcode {} not found for GPS", msg.header.op_code)
                ); // logs a warning (warn! is from the module log)
                Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Error: Opcode {} not found for GPS", msg.header.op_code),
                    //return a warning for NotFound opcode.
                ))
            }
        }
    }
}

fn main() {
    let log_path = "ex3_obc_fsw/handlers/gps_handler/logs";
    init_logger(log_path);
    trace!("Logger initialized")
    trace!("Starting GPS Handler...");

    // Create Unix domain socket interface to talk to message dispatcher
    let msg_dispatcher_interface = IpcServer::new("GPS".to_string());

    // Create IPC interface for GPS handler to talk to Comms (Messages for Ground Station)
    let gs_interface = IpcClient::new("gs_non_bulk".to_string());

    // example (TODO add gps_interface to GPSHandler object and poll in run loop)
    let mut gps_interface = IpcClient::new("gps_device".to_string()).ok();      // connect("/tmp/fifo_socket_gps_device")
    let _ = ipc_write(&gps_interface.as_ref().unwrap().fd, "time".as_bytes());  // send("time")
    thread::sleep(time::Duration::from_millis(100));                            // wait (only for example)
    let _ = poll_ipc_clients(&mut vec![&mut gps_interface]);                    // recv()
    println!("Got \"{}\"", String::from_utf8(gps_interface.as_mut().unwrap().read_buffer()).unwrap());
    
    let mut gps_handler = GPSHandler::new(msg_dispatcher_interface, gs_interface, gps_interface);
    
    let _ = gps_handler.run();
}
