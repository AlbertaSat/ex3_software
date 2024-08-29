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

use logging::*;
use log::{debug, trace, warn};
use common::component_ids::IRIS;
use common::opcodes::IRIS::GetHK;
use common::{opcodes, ports};
use ipc::*;
use message_structure::*;
use std::fs::OpenOptions;
use std::{io, thread};
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use std::time::{Instant, Duration};
use tcp_interface::*;
use std::collections::HashMap;
use serde_json::json;

const IRIS_DATA_DIR_PATH: &str = "ex3_obc_fsw/handlers/iris_handler/iris_data";
const IRIS_PACKET_SIZE: usize = 1252;
const IRIS_INTERFACE_BUFFER_SIZE: usize = IRIS_PACKET_SIZE;

/// Opcodes for messages relating to IRIS functionality
// pub enum OpCode {

// }

/// Interfaces are option types incase they are not properly created upon running this handler, so the program does not panic
struct IRISHandler {
    peripheral_interface: Option<TcpInterface>, // For communication with the IRIS peripheral [external to OBC]. Will be dynamic
    dispatcher_interface: Option<IpcClient>, // For communcation with other FSW components [internal to OBC] (i.e. message dispatcher)
}

impl IRISHandler {
    pub fn new(
        iris_interface: Result<TcpInterface, std::io::Error>,
        dispatcher_interface: Result<IpcClient, std::io::Error>,
    ) -> IRISHandler {
        //if either interfaces are error, print this
        if iris_interface.is_err() {
            warn!(
                "Error creating IRIS interface: {:?}",
                iris_interface.as_ref().err().unwrap()
            );
        }
        if dispatcher_interface.is_err() {
            warn!(
                "Error creating dispatcher interface: {:?}",
                dispatcher_interface.as_ref().err().unwrap()
            );
        }

        IRISHandler {

            peripheral_interface: iris_interface.ok(),
            dispatcher_interface: dispatcher_interface.ok(),
        }
    }

    fn handle_msg_for_iris(&mut self, msg: Msg) -> Option<String> {
        self.dispatcher_interface.as_mut().unwrap().clear_buffer();
        let mut hk = false;
        let op: String;
        let (command_msg, success) = match opcodes::IRIS::from(msg.header.op_code) {
            opcodes::IRIS::Reset=> {
                ("RST", true)
            }
            // Image commands
            opcodes::IRIS::ToggleSensor=> {
                if msg.msg_body[0] == 1 {
                    ("ON", true)
                } else if msg.msg_body[0] == 0 {
                    ("OFF", true)
                } else {
                    ("Error: invalid msg body for opcode 1", false)
                }
            }
            opcodes::IRIS::CaptureImage=> {
                ("TKI", true)
            }
            opcodes::IRIS::FetchImage=> {
                // Assumes that there are not more than 255 images being request at any one time
                op = format!("FTI:{}", msg.msg_body[0]);
                (op.as_str(), true)
            }
            opcodes::IRIS::GetImageSize=> {
                // Currently can only access the first 255 images stored on IRIS, will be updated if needed
                op = format!("FSI:{}", msg.msg_body[0]);
                (op.as_str(), true)
            }
            opcodes::IRIS::GetNImagesAvailable=> {
                ("FNI", true)
            }
            opcodes::IRIS::DelImage=> {
                op = format!("DTI:{}", msg.msg_body[0]);
                (op.as_str(),true)
            }
            // Housekeeping commands
            opcodes::IRIS::GetTime=> {
                ("FTT", true)
            }
            opcodes::IRIS::SetTime=> {
                // Placeholder for reading the total time need to determine how we will handle >255 values (ie. epoch time)
                op = format!("STT:{}", msg.msg_body[0]);
                (op.as_str(), true)
            }
            opcodes::IRIS::GetHK=> {
                hk = true;
                ("FTH", true)
            }
            opcodes::IRIS::Error => {
                op = format!("Opcode {} not found for IRIS", msg.header.op_code);
                (op.as_str(), false)
                
            }
        };
        if success {
            // Send command message to IRIS
            let status = TcpInterface::send(&mut self.peripheral_interface.as_mut().unwrap(), command_msg.as_bytes());

            if write_status(status, command_msg) { // Write succeeded
                let status = receive_response(self.peripheral_interface.as_mut().unwrap());
                
                match status {
                    // Ok(_data_len) => { println!("Got data {:?}", std::str::from_utf8(&response)); }
                    Ok(response) => {
                    trace!("Got data {:?}", response);
                        if hk {
                            return Some(response);
                        }
                    }
                    Err(e) => { debug!("Error: {}", e); }
                }

            }
            return None;
        }
        trace!("Command: {}", command_msg);
        None
    }
    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) -> std::io::Result<()> {

        // TMP 5 secs. More realistically every couple mins or so
        let hk_interval = Duration::from_secs(5);
        let mut last_hk_collect = Instant::now();

        // Read and poll for input for a message
        loop {

            // Check if we need to collect HK
            if last_hk_collect.elapsed() >= hk_interval {
                match self.collect_hk() {
                    Ok(_) => {
                        trace!("Collected and stored HK!");
                    }
                    Err(e) => {
                        debug!("HK collection failed: {}", e);
                    }
                }
                last_hk_collect = Instant::now();
            }

            // Sleep to prevent busy waiting
            // TODO - is ths necessary? What condition does this prevent? It works without sleep
            thread::sleep(Duration::from_millis(500));

            // Declare ipc interfaces we connect to
            let msg_dispatcher_interface = self.dispatcher_interface.as_mut().expect("Cmd_Msg_Disp has value of None");

            let mut clients = vec![
                msg_dispatcher_interface,
            ];
            poll_ipc_clients(&mut clients)?;
            
            // Handling the bulk message dispatcher interface
            if let Some(cmd_msg_dispatcher) = self.dispatcher_interface.as_mut() {
                if cmd_msg_dispatcher.buffer != [0u8; IPC_BUFFER_SIZE] {
                    let recv_msg: Msg = deserialize_msg(&cmd_msg_dispatcher.buffer).unwrap();
                    trace!("Received and deserialized msg");
                    self.handle_msg_for_iris(recv_msg);
                }
            }
        }
    }

    /// This function is a first iteration of how a handler will collect HK.
    /// Each handler will have a different version of this function as each HK is unique
    fn collect_hk(&mut self) -> io::Result<()> {
        let hk_msg = Msg::new(55,55,IRIS, IRIS, GetHK as u8, vec![]);
        if let Some(hk_string) = self.handle_msg_for_iris(hk_msg) {
            let hk_bytes = format_iris_hk(hk_string.as_bytes())?;
            store_iris_data("hk_test", &hk_bytes)?;
        }

        Ok(())
    }

    //TODO - Convert bytestream into message struct
    //TODO - After receiving the message, send a response back to the dispatcher
    //TODO - handle the message based on its opcode
}

/// Write IRIS data to a file (for now --- this may changer later if we use a db or other storage)
/// Later on we likely want to specify a path to specific storage medium (sd card 1 or 2)
/// We may also want to implement something generic to handle 'payload data' storage so we can have it duplicated, stored in multiple locations, or compressed etc.
fn store_iris_data(filename: &str, data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(IRIS_DATA_DIR_PATH)?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/{}", IRIS_DATA_DIR_PATH, filename))?;
    file.write_all(data)?;
    Ok(())
}

/// Format HK into JSON to create easily readable HK
/// 
fn format_iris_hk(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut hk_map = HashMap::new();
    
    // Convert data to string and trim newline characters
    let data_str = std::str::from_utf8(data).unwrap().trim_end();

    for line in data_str.lines() {
        if let Some((key, value)) = line.split_once(": ") {
            hk_map.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            debug!("Failed to process line of HK without ':' ");
        }
    }

    let json_value = json!(hk_map);
    let json_bytes = serde_json::to_vec(&json_value)?;
    trace!("Num HK bytes: {}", json_bytes.len());
    trace!("HK bytes: {:?}", json_bytes);
    
    Ok(json_bytes)
}


fn receive_response(peripheral_interface: &mut tcp_interface::TcpInterface) ->  Result<String, Error>{
    let mut packet_content = [0u8; IRIS_INTERFACE_BUFFER_SIZE];
    let packet_len = parse_packet(peripheral_interface, &mut packet_content, false,"None")?;
   
    if packet_len < 8 {}
    else if packet_content[0..7] == *"IMAGES:".as_bytes(){
        let mut n_images = 0;
        for i in 7..packet_len{
            n_images = n_images * 10 + packet_content[i]-48;
        }
        trace!("\nNum Images: {}\n", n_images);
        for _ in 0..n_images{
            parse_packet(peripheral_interface, &mut packet_content, false, "None")?;
            let status = std::str::from_utf8(&packet_content);
            let image: &str;
            match status {
                Ok(image_name) => { image = image_name.trim_matches(char::from(0)); }
                Err(_) => {  return Err(Error::new(ErrorKind::InvalidData, "image name improper")); }
            }
            // println!("{:?}", image);
            let mut image_success = [0u8; IRIS_INTERFACE_BUFFER_SIZE];
            parse_packet(peripheral_interface, &mut image_success, true, image)?;
            
        }
        return Ok("All images fetched".to_string());
    }
    let status = String::from_utf8(packet_content.to_vec());
    let response: String;
    match status {
        Ok(result) => { response = result.trim_matches(char::from(0)).to_string(); }
        Err(_) => {  return Err(Error::new(ErrorKind::InvalidData, "image name improper")); }
    }

    
    Ok(response)

}

/// Receives and translates IRIS packet, currently the IRIS simulated subsystem sends packets in the following format:
/// FLAG:length:...data...|END|, where length is replaced with the length of data
/// Until we know for certain the commands and their response structures this will have to make do
fn parse_packet(peripheral_interface: &mut tcp_interface::TcpInterface,  response:  &mut [u8; IRIS_INTERFACE_BUFFER_SIZE], is_image:  bool, image_name: &str) ->  Result<usize, Error>{
    let flag: [u8; 4] = [70, 76, 65, 71]; // is "FLAG" in bytes
    let mut flag_match: usize = 0;
    let delim = 58; // is the delimiter for the simulated subsystem ":"

    // Packet buffers
    let mut packet_length: usize = 0;
    let mut packet_byte = vec![0u8; 1]; // For reading one byte at a time
    let mut packet_buffer = vec![0u8; IRIS_INTERFACE_BUFFER_SIZE]; // For reading a full packet

    
    // Check for flag
    while flag_match < flag.len() {
        TcpInterface::read(peripheral_interface, &mut packet_byte)?;
        if packet_byte[0] == flag[flag_match]{
            flag_match = flag_match + 1;
        }
        else { flag_match = 0; }
    }
    TcpInterface::read(peripheral_interface, &mut packet_byte)?; // Consume delimiter
    TcpInterface::read(peripheral_interface, &mut packet_byte)?; // Read first int of packet length
    

    // Get packet length
    while packet_byte[0] != delim {
        packet_length = (packet_length*10) + ((packet_byte[0]-48) as usize); // Increase packet length  (Assumes there is a present for packet length)    
        TcpInterface::read(peripheral_interface, &mut packet_byte)?;
    }

    // Read packet, currently only images are > 1 packet
    if is_image {
        let mut temp_length = packet_length;
        while temp_length > IRIS_INTERFACE_BUFFER_SIZE{ // Read in full packets
            TcpInterface::read(peripheral_interface, &mut packet_buffer)?;
            store_iris_data(image_name, &packet_buffer)?;
            temp_length = temp_length - IRIS_INTERFACE_BUFFER_SIZE;
        }
        for index in 0..temp_length { // Read the final partial packet
            TcpInterface::read(peripheral_interface, &mut packet_byte)?; 
            packet_buffer[index] = packet_byte[0];
        }
        store_iris_data(image_name, &packet_buffer)?; // Append packet to image file
    }
    else {
        for index in 0..packet_length {
            TcpInterface::read(peripheral_interface, &mut packet_byte)?; 
            response[index] = packet_byte[0];
            // print!("{}", packet_byte[0] as char);
        }
    }

    return Ok(packet_length);

}

/// Verify that a command was successfully sent via checking the return status of a tcp write
/// Returns either true on successful send or false on failed send
fn write_status(status: Result<usize, Error>, cmd: &str) -> bool{
    match status {
        Ok(_data_len) => {
            trace!("Command {} successfully sent", cmd);
            true
        }
        Err(e) => {
            debug!("Error: {}", e);
            false
        }
    }
}

fn main() {
    //For now interfaces are created and if their associated ports are not open, they will be ignored rather than causing the program to panic

    //Create TCP interface for IRIS handler to talk to simulated IRIS
    let iris_interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_IRIS_PORT);

    //Create IPC interface for IRIS handler to talk to message dispatcher
    let dispatcher_interface = IpcClient::new("iris_handler".to_string());

    //Create IRIS handler
    let mut iris_handler = IRISHandler::new(iris_interface, dispatcher_interface);

    // Initialize logging
    let log_path = "ex3_obc_fsw/handlers/iris_handler/logs";
    init_logger(log_path);
    
    trace!("Beginning IRIS Handler...");
    let _ = iris_handler.run();
}
