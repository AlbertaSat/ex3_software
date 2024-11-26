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
use common::opcodes;
use ipc::{IpcClient, IpcServer, IPC_BUFFER_SIZE, ipc_write, poll_ipc_clients, poll_ipc_server_sockets};
use message_structure::*;
use std::{thread, time};

const HK_INTERVAL = time::Duration::from_secs(20);

/* Comments from Hari:
"Why should the GPS ever directly communicate to the ground station? No reason. 
The gps isnt on the BUS. The gps isnt connected to the spacecraft bus, It HAS to go to the OBC
Talk to OBC over PHYSICAL port.
The GPS shouldnt decide where the data goes. ."
*/

struct GPSHandler {
    msg_dispatcher_interface: Option<IpcServer>, // For communcation with other FSW components [internal to OBC]
    gs_interface: Option<IpcClient> // For sending messages to the GS through the coms_handler
    gps_interface: Option<IpcClient> // For sending messages to the GPS 
}

impl GPSHandler {
    pub fn new( 
        msg_dispatcher_interface: Result<IpcServer, std::io::Error>,
        gs_interface: Result<IpcServer, std::io::Error>,
        gps_interface: Result<IpcServer, std::io::Error>,
    ) -> GPSHandler {
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

        //Note that HK_INTERVAL is a global const
        let mut last_hk_collect = time::Instant::now(); // Housekeeping should occur regularly. Begin the timer now.

        // Poll for messages
        loop {

            // Check if we should collect HK:
            if last_hk_collect.elapsed() >= HK_INTERVAL {
                match self.collect_hk() {
                    Ok(_) => {
                        debug!("Collected and stored HK.");
                    }
                    Err(e) => {
                        debug!("HK collection failed: {}", e);
                    }
                }
                last_hk_collect= time::Instant::now();
            }

            // First, take the Option<IpcClient> out of `self.dispatcher_interface`
            // This consumes the Option, so you can work with the owned IpcClient
            let msg_dispatcher_interface = self.msg_dispatcher_interface.take().expect("Cmd_Disp has value of None");

            // Create a mutable Option<IpcClient> so its lifetime persists
            let mut msg_dispatcher_interface_option = Some(msg_dispatcher_interface);

            // Now you can borrow this mutable option and place it in the vector
            let mut server: Vec<&mut Option<IpcServer>> = vec![
                &mut msg_dispatcher_interface_option,
                //QUESTION: do we need to add the other interfaces here?
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


        /// handles how GPS will collect HK
        /// most of this is placeholder as we do not yet know what kind of HK data to recieve
        fn collect_hk(&mut self) -> io::Result<()> {
            let hk_msg = Msg::new("HK_test_one".to_string()) //Question: idk what to put in it now, but will need to make a Msg for Hk...
            if let Some(hk_string) = self.handle_msg_for_gps(hk_msg) {
                let hk_bytes = format_gps_hk(hk_string.as_bytes())?;
                store_gps_data("HK_test", &hk_bytes)?;
            }

            ok(());
            // TODO: WHAT IS GPS HK ACTION
            //UNFINISHED
        }

    }

    fn handle_msg_for_gps(&mut self, msg: Msg) -> Result<(), Error> {
        //match the opcodes with the message header op_code
        //returns none if Ok, Error if err
        self.msg_dispatcher_interface.as_mut().unwrap().clear_buffer(); //Question: why this line?
        println!("GPS msg opcode: {} {:?}", msg.header.op_code, msg.msg_body);
        // handle opcodes: https://docs.google.com/spreadsheets/d/1rWde3jjrgyzO2fsg2rrVAKxkPa2hy-DDaqlfQTDaNxg/edit?gid=0#gid=0
        match opcodes::GPS::from(msg.header.op_code){
            //for now im using the simulated gps commands but this will change when we get the actual gps commands
            opcodes::GPS::GetLatLong => {   
                trace!("Getting latitude and longitude");
                // QUESTION: what to put here
                // steps:
                //  get data from GPS; 
                //  if data < 128, send to GS else send to bulk
            }
            opcodes::GPS::GetUTCTime => {
                trace!("Getting UTC time");

            }
            opcodes::GPS::GetHK => {
                trace!("Getting HK");
            
            }
            opcodes::GPS::Reset => {
                trace!("Resetting");
                //TODO: WE DONT HAVE RESET ON THE SIM GPS RN...   
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


    /// Format HK into a JSON to create an easily readable HK
    /// copied from iris handler
    fn format_gps_hk(data: &[u8]) - > Result<Vec<u8>>, std::io::Error> {
        let mut hk_map = HashMap::new() // think of hashmap as python dict

        //  convert data to string and trim newline characters
        let data_str = std::str::from_utf8(data).unwrap().trim_end();

        for line in data_str.lines() {
            //Some tells us we know a value is present; in this case a key value pair
            if let Some((key, value)) = line.split_once(": ") {     
                hk_map.insert(key.trim().to_string(), value.trim().to_string());
            } else {
                debug!("Failed to precess line of HK without ':' ");
            }
        }

        let json_value = json!(hk_map);
        let json_bytes = serde_json::to_vec(&json_value)?;
        race!("Num HK bytes: {}", json_bytes.len());
        trace!("HK bytes: {:?}", json_bytes);
    
        Ok(json_bytes)
    }

    fn write_msg_to_GS(interface: &mut TcpInterface, msg: MSG) {
        // heavily lifted from coms_handler
        let serialized_msg_result = serialize_msg(&msg);    // converts msg to bytes -> Result<Vec<u8>
        match serialized_msg_result {
            Ok(serialized_msg) => {
                // TODO
            }
            Err(e) => {
                debug!("Error sending message to GS");

            }
        }

    }
}

fn main() {
    // Initialize logging
    let log_path = "ex3_obc_fsw/handlers/gps_handler/logs";
    init_logger(log_path);
    trace!("Logger initialized")
    trace!("Starting GPS Handler...");

    // Create Unix domain socket interface to talk to message dispatcher
    let msg_dispatcher_interface = IpcServer::new("GPS".to_string());

    // Create IPC interface for GPS handler to talk to Comms (Messages for Ground Station)
    let gs_interface = IpcClient::new("gs_non_bulk".to_string());

    // Create IPC interface for GPS handler to talk to simulated GPS 
    let gps_interface = IpcClient::new("gps_device".to_string());   // connect("/tmp/fifo_socket_gps_device")
    
    // Create GPS handler
    let mut gps_handler = GPSHandler::new(msg_dispatcher_interface, gs_interface, gps_interface);
    
    /*
    Below is example written by Kaaden:
// example (TODO add gps_interface to GPSHandler object and poll in run loop)
let mut gps_interface = IpcClient::new("gps_device".to_string()).ok();      // connect("/tmp/fifo_socket_gps_device")
let _ = ipc_write(&gps_interface.as_ref().unwrap().fd, "time".as_bytes());  // send("time")
thread::sleep(time::Duration::from_millis(100));                            // wait (only for example)
let _ = poll_ipc_clients(&mut vec![&mut gps_interface]);                    // recv()
println!("Got \"{}\"", String::from_utf8(gps_interface.as_mut().unwrap().read_buffer()).unwrap()); */

    // Start the GPS handler
    match gps_handler.run() {
        Ok(_) => debug!("GPS handler run successfully!"),
        Err(e) => debug!("Error occured while running GPS handler: {}", e),
    }
}
