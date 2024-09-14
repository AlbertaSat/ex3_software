

/*
Written by Drake Boulianne
Summer 2024

This module contains functions for handling the UHF (UHF simulated as of now). It consists mainly of 
getting and setting functions for the simulated UHF parameters. Each getter or setter will return a
vector of bytes containing the data returned from the UHF. The only public function in the module is the 
handle_uhf_cmd, Which takes the tcp uhf_interface and message as arguments, Then returns a vector of bytes containing
the data returned from the UHF after the request or an error message if the request times out.
*/
use tcp_interface::{Interface, TcpInterface};
use message_structure::*;
use common::opcodes;
use common::constants::UHF_MAX_MESSAGE_SIZE_BYTES;

// Struct containing UHF parameters to be modified
pub struct UHFHandler {
    mode: u8,
    beacon: String,
}

// Implementations (getters and setters) 
impl UHFHandler {
    pub fn new() -> UHFHandler {
        // This will eventually actually talk to the UHF and grab the parameters it currently has, for now just dummy values
        UHFHandler {
            mode: 0,
            beacon: String::from("Beacon"),
        }
    }
    pub fn handle_msg_for_uhf(&mut self, uhf_interface: &mut TcpInterface, msg: &Msg) {
        // Can Only use this function when we have simulated UHF integrated with rest of OBC software
        let opcode = opcodes::UHF::from(msg.header.op_code);
        let content = msg.clone().msg_body;
        match opcode {
            opcodes::UHF::GetHK => {
                self.get_hk_data()
            },
            opcodes::UHF::SetBeacon => {
                self.set_beacon_value(uhf_interface, content);
            },
            opcodes::UHF::GetBeacon => {
                self.get_beacon_value();
            },
            opcodes::UHF::SetMode => {
                self.set_mode(uhf_interface, content);
            },
            opcodes::UHF::GetMode => {
                self.get_mode();
            }
            opcodes::UHF::Reset => {
                self.reset_uhf();
            },
            _ => {
                println!("Invalid opcode");
            }
        }
    }


    fn set_beacon_value(&mut self, uhf_interface: &mut TcpInterface, new_beacon_value: Vec<u8>) {
        // Construct command for simulated UHF
        let prefix: Vec<u8> = "UHF:SET_BEACON:".as_bytes().to_vec();
        let mut cmd: Vec<u8> = new_beacon_value.clone();
        cmd.splice(0..0, prefix);
    
        //Send the command
        send_msg(uhf_interface, cmd);
        // Read Buffer to effectively clear it.
        let  _ = read_buffer(uhf_interface);

        let beacon = String::from_utf8(extract_non_null_bytes(new_beacon_value)).unwrap();
        println!("Set Beacon value to {}", &beacon);
        self.beacon = beacon;
    }   


    fn get_beacon_value(&self) {
        println!("Current UHF Beacon Message: {}", self.beacon);
    }
    
    
    fn set_mode(&mut self, uhf_interface: &mut TcpInterface, new_mode: Vec<u8>) {


        // Create Command.
        let prefix: Vec<u8> = "UHF:SET_MODE:".as_bytes().to_vec();
        let mut cmd: Vec<u8> = new_mode.clone();
        cmd.splice(0..0, prefix);
    
        // Send Command.
        send_msg(uhf_interface, cmd);
        // Read Buffer to effectively clear it.
        let  _ = read_buffer(uhf_interface);
        self.mode = new_mode[0].clone();
        println!("UHF Mode Set to: {}", self.mode);
 
    }
    

    fn get_mode(&self) {
        println!("Current UHF Mode: {}", self.mode);
    }
    

    fn get_hk_data(&self) {
        println!("Getting HK Data");
    }
    

    fn reset_uhf(&self) {
        println!("Resetting UHF");
    }
}


fn read_buffer(uhf_interface: &mut TcpInterface) -> Vec<u8> {
    let mut buffer: Vec<u8> = vec![0; UHF_MAX_MESSAGE_SIZE_BYTES as usize]; //Buffer to read incoming messages from UHF
    let read_result: Result<usize, std::io::Error> = TcpInterface::read(uhf_interface, &mut buffer);
    match read_result {
        Ok(_n) => {
            buffer.to_vec()
        }, 
        Err(_) => {
            "Failed to read".as_bytes().to_vec()
        }
    }
}


fn send_msg(uhf_interface: &mut TcpInterface, content: Vec<u8>) {
    let send_result = uhf_interface.send(&content);
    match send_result {
        Ok(_) => println!("Send successful."),
        Err(e) => println!("Error occured setting beacon value:  {:?}", e)
    }
}

pub fn extract_non_null_bytes(buffer: Vec<u8>) -> Vec<u8> {
    // function used for testing. Takes vector of bytes and returns array without null characters. 
    let mut useful_msg: Vec<u8> = Vec::new();
    for byte in buffer {
        if byte == b'\0' {
            continue;
        } else {
            useful_msg.push(byte)
        }
    }
    useful_msg
}
