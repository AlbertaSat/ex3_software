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
use ipc::*;
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

            peripheral_interface: iris_interface.ok(),
            dispatcher_interface: dispatcher_interface.ok(),
        }
    }

    fn handle_msg_for_iris(&mut self, msg: Msg){
        self.dispatcher_interface.as_mut().unwrap().clear_buffer();
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
                (&*format!("FTI:{}", msg.msg_body[0]),true)
            }
            opcodes::IRIS::GetImageSize=> {
                // Currently can only access the first 255 images stored on IRIS, will be updated if needed
                (&*format!("FSI:{}", msg.msg_body[0]),true)
            }
            opcodes::IRIS::GetNImagesAvailable=> {
                ("FNI", true)
            }
            opcodes::IRIS::DelImage=> {
                (&*format!("DTI:{}", msg.msg_body[0]),true)
            }
            // Housekeeping commands
            opcodes::IRIS::GetTime=> {
                ("FTT", true)
            }
            opcodes::IRIS::SetTime=> {
                // Placeholder for reading the total time need to determine how we will handle >255 values (ie. epoch time)
                (&*format!("STT:{}", msg.msg_body[0]),true)
            }
            opcodes::IRIS::GetHK=> {
                ("FTH", true)
            }
            opcodes::IRIS::Error => {
                (&*format!("Opcode {} not found for IRIS", msg.header.op_code), false)
                
            }
        };
        if success {
            // Send command message to IRIS
            let status = TcpInterface::send(&mut self.peripheral_interface.as_mut().unwrap(), command_msg.as_bytes());

            if write_status(status, command_msg) { // Write succeeded
                let status = receive_response(self.peripheral_interface.as_mut().unwrap());
                
                match status {
                    // Ok(_data_len) => { println!("Got data {:?}", std::str::from_utf8(&response)); }
                    Ok(response) => { println!("Got data {:?}", response); }
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
        loop {
            let msg_dispatcher_interface = self.dispatcher_interface.as_mut().expect("Cmd_Msg_Disp has value of None");

            let mut clients = vec![
                msg_dispatcher_interface,
            ];
            poll_ipc_clients(&mut clients)?;
            
            // Handling the bulk message dispatcher interface
            if let Some(cmd_msg_dispatcher) = self.dispatcher_interface.as_mut() {
                if cmd_msg_dispatcher.buffer != [0u8; IPC_BUFFER_SIZE] {
                    let recv_msg: Msg = deserialize_msg(&cmd_msg_dispatcher.buffer).unwrap();
                    println!("Received and deserialized msg");
                    self.handle_msg_for_iris(recv_msg);
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
fn store_iris_data(filename: &str, data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(IRIS_DATA_DIR_PATH)?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/{}", IRIS_DATA_DIR_PATH, filename))?;
    file.write_all(data)?;
    Ok(())
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
        print!("\n{}\n", n_images);
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

    
    return Ok(response);

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
    let dispatcher_interface = IpcClient::new("iris_handler".to_string());

    //Create IRIS handler
    let mut iris_handler = IRISHandler::new(iris_interface, dispatcher_interface);

    let _ = iris_handler.run();
}
