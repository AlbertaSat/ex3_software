use std::fs::read;

/*
Written by Drake Boulianne
Summer 2024

This module contains functions for handling the UHF (UHF simulated as of now). It consists mainly of 
getting and setting functions for the simulated UHF parameters. Each getter or setter will return a
vector of bytes containing the data returned from the UHF. The only public function in the module is the 
handle_uhf_cmd, Which takes the tcp interface and message as arguments, Then returns a vector of bytes containing
the data returned from the UHF after the request or an error message if the request times out.
*/
use tcp_interface::{Interface, TcpInterface};
use message_structure::Msg;
use common::opcodes;
use common::constants::UHF_MAX_MESSAGE_SIZE_BYTES;
use common::ports;
use common::component_ids::ComponentIds::UHF;



fn set_beacon_value(interface: &mut TcpInterface, new_beacon_value: Vec<u8>) -> Vec<u8>{
    // TODO - write this data to the UHF beacon buffer (or however it actually works w/ the hardware)
    let prefix: Vec<u8> = "UHF:SET_BEACON:".as_bytes().to_vec();

    // Temporary command format sent to UHF to modify beacon 
    let mut content: Vec<u8> = new_beacon_value.clone();
    content.splice(0..0, prefix);

    // Send message to UHF
    let send_result = interface.send(&content);
    match send_result {
        Ok(_) => println!("Send successful."),
        Err(e) => println!("Error occured setting beacon value:  {:?}", e)
    }
    // Then get and return beacon value to confirm it was changed
    get_beacon_value(interface)
}

fn get_beacon_value(interface: &mut TcpInterface) -> Vec<u8>{

    let request = "UHF:GET_BEACON:".as_bytes().to_vec();
    let mut buffer: Vec<u8> = Vec::new();
    // Send message to UHF
    let send_result = interface.send(&request);
    match send_result {
        Ok(_) => {},
        Err(e) => println!("Error occured getting beacon value: {:?}", e)
    }
    // loop awaiting for message. TODO: This should timeout after a while...
    loop {
        if let Ok(n) = TcpInterface::read(interface, &mut buffer) {
            if n > 0 {
                return buffer
            } else {
                continue;
            }
        } else {
            println!("No bytes to read");
        }
    }
}

fn set_mode(interface: &mut TcpInterface, new_mode: Vec<u8>) -> Vec<u8> {
    let prefix: Vec<u8> = "UHF:SET_MODE:".as_bytes().to_vec();

    let mut content: Vec<u8> = new_mode.clone();
    content.splice(0..0, prefix);

    // Send message to UHF
    let send_result = interface.send(&content);
    match send_result {
        Ok(_) => println!("Send successful."),
        Err(e) => println!("Error occured setting beacon value:  {:?}", e)
    }
    get_mode(interface)
}

fn get_mode(interface: &mut TcpInterface) -> Vec<u8> {
    let request = "UHF:GET_MODE:".as_bytes().to_vec();
    let mut buffer: Vec<u8> = Vec::new();
    // Send message to UHF
    let send_result = interface.send(&request);
    match send_result {
        Ok(_) => {},
        Err(e) => println!("Error occured getting beacon value: {:?}", e)
    }
    // loop awaiting for message. TODO: This should timeout after a while...
    loop {
        if let Ok(n) = TcpInterface::read(interface, &mut buffer) {
            if n > 0 {
                return buffer
            } else {
                continue;
            }
        } else {
            println!("No bytes to read");
        }
    }
}


fn set_baud_rate(interface: &mut TcpInterface, new_baud_rate: Vec<u8>) -> Vec<u8>{
    let prefix: Vec<u8> = "UHF:SET_MODE:".as_bytes().to_vec();

    let mut content: Vec<u8> = new_baud_rate.clone();
    content.splice(0..0, prefix);

    // Send message to UHF
    let send_result = interface.send(&content);
    match send_result {
        Ok(_) => println!("Send successful."),
        Err(e) => println!("Error occured setting beacon value:  {:?}", e)
    }
    get_baud_rate(interface)
}


fn get_baud_rate(interface: &mut TcpInterface) -> Vec<u8> {
    let request = "UHF:GET_BAUD_RATE:".as_bytes().to_vec();
    let mut buffer: Vec<u8> = Vec::new();
    // Send message to UHF
    let send_result = interface.send(&request);
    match send_result {
        Ok(_) => {},
        Err(e) => println!("Error occured getting beacon value: {:?}", e)
    }
    // loop awaiting for message. TODO: This should timeout after a while...
    loop {
        if let Ok(n) = TcpInterface::read(interface, &mut buffer) {
            if n > 0 {
                return buffer
            } else {
                continue;
            }
        } else {
            println!("No bytes to read");
        }
    }
}


fn get_hk_data(interface: &mut TcpInterface, content: Vec<u8>) -> Vec<u8> {
    // Just returns "NO HK DATA" for now
    vec![0x4E, 0x4F, 0x20, 0x48, 0x4B, 0x20, 0x44, 0x41, 0x54, 0x41]
}

fn reset_uhf(interface: &mut TcpInterface) -> Vec<u8> {
    let bytes: Vec<u8> = vec![
    0x52, 0x65, 0x73, 0x65, 0x74, 0x74, 0x69, 0x6E, 0x67, 0x20, 
    0x55, 0x48, 0x46, 0x2E, 0x2E, 0x2E
    ];
    bytes


}

pub fn handle_uhf_cmd(interface: &mut TcpInterface, msg: &Msg) -> Vec<u8>{
    let opcode = opcodes::UHF::from(msg.header.op_code);
    
    match opcode {
        opcodes::UHF::GetHK => {
            get_hk_data(interface, msg.msg_body.clone())
        },
        opcodes::UHF::SetBeacon => {
            set_beacon_value(interface, msg.msg_body.clone())
        },
        opcodes::UHF::GetBeacon => {
            get_beacon_value(interface)
        },
        opcodes::UHF::SetBaudRate => {
            set_baud_rate(interface, msg.msg_body.clone())
        },
        opcodes::UHF::GetBaudRate => {
            get_baud_rate(interface)
        },
        opcodes::UHF::Reset => {
            reset_uhf(interface)
        },
        _ => {
            println!("Invalid opcode");
            // return error message if opcode is invalid
            String::from("ERR - INVALID OPCODE").into_bytes()
        }
    }

}



#[test]
fn test_setting() {
    /*
    This is a test for the functionality of setting simulated parameters on the simulated UHF.
    In this test we start running the test_setting.sh script which fires up the simulated UHF and a GS terminal.
    Next the test_setting function initiates the Tcp interface and creates new simulated parameters to be sent as strings.
    For each simulated parameter to be set, a msg is constructed, the command is then handled, and then the buffer
    is read into a variable. an assert statement then checks the values within the vector returned
    
     */


    let mut interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_COMMS_PORT).unwrap();
    let new_beacon: String = String::from("bacon");
    let new_mode: String = String::from("9");
    let new_baud_rate: String = String::from("4550");

    // Test setting beacon value
    let msg: Msg = Msg::new(0, 0, UHF.into(), UHF.into(), 200, new_beacon.clone().into_bytes());
    let returned: Vec<u8> = handle_uhf_cmd(&mut interface, &msg);
    assert_eq!(new_beacon, String::from_utf8(returned).unwrap());

    // Test setting mode
    let msg: Msg = Msg::new(0, 0, UHF.into(), UHF.into(), 200, new_mode.clone().into_bytes());
    let returned: Vec<u8> = handle_uhf_cmd(&mut interface, &msg);
    assert_eq!(new_mode, String::from_utf8(returned).unwrap());

    // Test setting baud rate
    let msg: Msg = Msg::new(0, 0, UHF.into(), UHF.into(), 200, new_baud_rate.clone().into_bytes());
    let returned: Vec<u8> = handle_uhf_cmd(&mut interface, &msg);
    assert_eq!(new_baud_rate, String::from_utf8(returned).unwrap());


}