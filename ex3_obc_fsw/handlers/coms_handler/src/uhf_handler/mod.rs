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
use log::{trace, warn, debug};

// Struct containing UHF parameters to be modified
pub struct UHFHandler {
    mode: u8,
    beacon: String,
    buffer: Vec<u8>
}

// Implementations (getters and setters) 
impl UHFHandler {
    pub fn new() -> UHFHandler {
        // This will eventually actually talk to the UHF and grab the parameters it currently has, for now just dummy values
        UHFHandler {
            mode: 0,
            beacon: String::from("Beacon"),
            buffer: vec![0; UHF_MAX_MESSAGE_SIZE_BYTES as usize],
        }
    }
    pub fn handle_msg_for_uhf(&mut self, uhf_interface: &mut TcpInterface, msg: &Msg) {
        // Can Only use this function when we have simulated UHF integrated with rest of OBC software
        let opcode = opcodes::UHF::from(msg.header.op_code);
        let data = msg.msg_body.clone();
        match opcode {
            opcodes::UHF::GetHK => {
                self.get_hk_data()
            },
            opcodes::UHF::SetBeacon => {
                self.set_beacon_value(uhf_interface, data);
            },
            opcodes::UHF::GetBeacon => {
                self.get_beacon_value(uhf_interface);
            },
            opcodes::UHF::SetMode => {
                self.set_mode(uhf_interface, data);
            },
            opcodes::UHF::GetMode => {
                self.get_mode(uhf_interface);
            }
            opcodes::UHF::Reset => {
                self.reset_uhf();
            },
            _ => {
                println!("Invalid opcode");
            }
        }
        // clear uhf buffer after command is handled
        self.clear_buffer();

    }


    fn set_beacon_value(&mut self, uhf_interface: &mut TcpInterface, data: Vec<u8>) {
        // Extract useful bytes from data
        let new_beacon_as_bytes = extract_non_null_bytes(data);
        // Beacon bytes can only be ASCII encoded letters or numbers, if other return early
        for ascii_byte in &new_beacon_as_bytes {
            if is_valid_ascii_digit_or_letter(*ascii_byte) {
                continue;
            } else {
                warn!("Byte {}, is not a valid ascii encoded digit or letter.", *ascii_byte as char);
                return;
            }
        }
        // Check if data can be converted to UTF-8, return early if not able to
        let new_beacon_as_string = match String::from_utf8(new_beacon_as_bytes.clone()) {
            Ok(beacon_str) => beacon_str,
            Err(e) => {
                warn!("Error converting bytes to UTF-8: {}", e);
                warn!("Abort setting beacon value.");
                return;
            }
        };
        // Construct command for simulated UHF if new beacon string is okay
        let prefix: Vec<u8> = "UHF:SET_BEACON:".as_bytes().to_vec();
        let mut cmd: Vec<u8> = new_beacon_as_bytes;
        cmd.splice(0..0, prefix);
    
        //Send the command
        self.send_msg(uhf_interface, cmd);
        // Read Buffer uhf buffer, in case we want to use this message later for now we just clear it after read.
        self.read_into_buffer(uhf_interface);
        self.clear_buffer();


        trace!("Set Beacon value to {}", &new_beacon_as_string);
        self.beacon = new_beacon_as_string;
    }   


    fn get_beacon_value(&mut self, uhf_interface: &mut TcpInterface) {
        // construct command to get UHF beacon
        let cmd: Vec<u8> = "UHF:GETBEACON:".as_bytes().to_vec();
        // send command 
        self.send_msg(uhf_interface, cmd);
        // read response from UHF
        self.read_into_buffer(uhf_interface);
        // convert response to string, return early if it fails
        let response = match String::from_utf8(self.buffer.clone()) {
            Ok(response) => response,
            Err(e) => {
                warn!("Error parsing response from UHF. Could not get beacon value from UHF: {}", e);
                return;
            }
        };
        if response == self.beacon {
            trace!("Current UHF Beacon Message: {}", self.beacon);
        } else {
            warn!("UHF beacon and UHF handler beacon values are out of sync");
            warn!("UHF: {} | UHF handler {}", response, self.beacon)
        }
    }
    
    
    fn set_mode(&mut self, uhf_interface: &mut TcpInterface, data: Vec<u8>) {
        // Extract useful bytes from data
        let new_mode_as_bytes = extract_non_null_bytes(data);
        for ascii_byte in &new_mode_as_bytes {
            if is_valid_ascii_digit(*ascii_byte) {
                continue;
            } else {
                warn!("Byte {}, is not a valid ascii encoded digit. ", *ascii_byte as char);
                warn!("Abort setting mode value.");
                return;
            }
        }
        // Check if data can be converted to UTF-8, return early if not able to
        let new_mode_as_string = match String::from_utf8(new_mode_as_bytes.clone()) {
            Ok(mode_str) => mode_str,
            Err(e) => {
                warn!("Error converting bytes to UTF-8: {}", e);
                warn!("Abort setting mode value.");
                return;
            }
        };

        let new_mode_as_u8: u8 = match new_mode_as_string.parse() {
            Ok(new_mode) => new_mode,
            Err(e) => {
                warn!("Error occured parsing mode into integer: {e}");
                warn!("Aborting setting mode value");
                return
            }
        };
        // Create Command.
        let prefix: Vec<u8> = "UHF:SET_MODE:".as_bytes().to_vec();
        // Remove extra bytes from the new beacon value msg
        let mut cmd: Vec<u8> = new_mode_as_bytes;
        cmd.splice(0..0, prefix);
    
        // Send Command.
        self.send_msg(uhf_interface, cmd);
        // Read Buffer uhf buffer, in case we want to use this message later for now we just clear it after read.
        self.read_into_buffer(uhf_interface);
        self.clear_buffer();
        self.mode = new_mode_as_u8;
        trace!("UHF Mode Set to: {}", self.mode); 
    }
    

    fn get_mode(&mut self, uhf_interface: &mut TcpInterface) {
        // construct command to get UHF beacon
        let cmd: Vec<u8> = "UHF:GETMODE:".as_bytes().to_vec();
        // send command 
        self.send_msg(uhf_interface, cmd);
        // read response from UHF
        self.read_into_buffer(uhf_interface);
        // convert response to string, return early if it fails
        let response = match String::from_utf8(self.buffer.clone()) {
            Ok(response) => response,
            Err(e) => {
                warn!("Error parsing response from UHF. Could not get beacon value from UHF: {}", e);
                return;
            }
        };
        if response == self.beacon {
            println!("Current UHF Beacon Message: {}", self.beacon);
        } else {
            warn!("UHF beacon and UHF handler beacon values are out of sync");
            warn!("UHF: {} | UHF handler {}", response, self.beacon)
        }
    }
    

    fn get_hk_data(&self) {
        trace!("Getting HK Data");
    }
    

    fn reset_uhf(&self) {
        trace!("Resetting UHF");
    }

    fn read_into_buffer(&mut self, uhf_interface: &mut TcpInterface) {
        // read bytes into UHF buffer
        let read_result: Result<usize, std::io::Error> = TcpInterface::read(uhf_interface, &mut self.buffer);
        match read_result {
            Ok(n) => {
                trace!("Read {} bytes from uhf", n)
            }, 
            Err(_) => {
                debug!("Error reading bytes from UHF")
            }
        }
    }
    
    
    fn send_msg(&mut self, uhf_interface: &mut TcpInterface, content: Vec<u8>) {
        let send_result = uhf_interface.send(&content);
        match send_result {
            Ok(_) => trace!("Send successful."),
            Err(e) => warn!("Error occured setting beacon value:  {:?}", e)
        }
    }

    fn clear_buffer(&mut self) {
        self.buffer.fill(0);
    }
}



pub fn extract_non_null_bytes(buffer: Vec<u8>) -> Vec<u8> {
    // Takes vector of bytes and returns vector without null characters.
    // This means that this function assumes that the data is encoded in unicode
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

fn is_valid_ascii_digit_or_letter(byte: u8) -> bool {
    // checks if byte is a valid base 10 ascii encoded letter or digit
    match byte {
        48..=57  
        | 65..=90 
        | 97..=122 => true, 
        _ => false,  
    }
}

fn is_valid_ascii_digit(byte: u8) -> bool {
    // checks if byte is a valid base 10 ascii encoded digit
    match byte {
        48..=57 => true,
        _ => false
    }
}