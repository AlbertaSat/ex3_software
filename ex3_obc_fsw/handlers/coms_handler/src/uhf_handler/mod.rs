use std::fs::read;

/*
Written by Drake Boulianne
Summer 2024

This program is a handler for the UHF (Currently UHF is only simulated). It creates a TCP client connection with
the Comms handler side TCP server. The UHF handler is capable of transmitting, and receiving messages to the UHF.
Its main purpose is to modify operating parameters of the UHF (Ex: baud rate). Its TCP connection is versatile and
allows for the UHF handler program to disconnect from the UHF and continue to look for a connection. 
*/
use tcp_interface::{Interface, TcpInterface};
use message_structure::{deserialize_msg, serialize_msg, Msg};
use common::opcodes;


fn set_beacon_value(interface: &mut TcpInterface, new_beacon_value: Vec<u8>) {
    // TODO - write this data to the UHF beacon buffer (or however it actually works w/ the hardware)
    println!("Setting beacon value to: {:?}", new_beacon_value);
    let prefix = "MOD_UHF:".as_bytes().to_vec();

    // Temporary command format sent to UHF to modify beacon 
    let mut content = new_beacon_value.clone();
    content.splice(0..0, prefix);

    // Send message to UHF
    let send_result = interface.send(&content);
    match send_result {
        Ok(_) => println!("Successfully set beacon to {:?}", new_beacon_value),
        Err(e) => println!("Error occured setting beacon value:  {:?}", e)
    }
}

fn get_beacon_value(interface: &mut TcpInterface) {
    // TODO - write this data to the UHF beacon buffer (or however it actually works w/ the hardware)
    // Place holder messsage to be sent to the UHF to grab beacon
    let request = "MOD_UHF:GET_BEACON".as_bytes().to_vec();

    // Send message to UHF
    let send_result = interface.send(&request);
    match send_result {
        Ok(_) => println!("Successfully requested value of beacon"),
        Err(e) => println!("Error occured getting beacon value: {:?}", e)
    }
    let mut buffer: Vec<u8> = Vec::new();
    loop {
        if let Ok(n) = TcpInterface::read(interface, &mut buffer) {
            println!("got dem bytes: {:?}", buffer);
            if n > 0 {
                break;
            } else {
                continue;
            }
        } else {
            println!("No bytes to read");
        }
    }
}


fn get_baud_rate(interface: &mut TcpInterface) {
    /*
    This is a getter function to get the UHF's baud rate.
    Args:
        None
    Returns:
        baud_rate(u64): Current Baud rate of the UHF
     */
    let baud_rate = 9600;
    println!("Getting UHF baud rate... ");
    println!("{}", baud_rate)
}

fn set_baud_rate(interface: &mut TcpInterface, new_baud_rate: Vec<u8>) {
    /*
    This function sets the baud rate for the UHF
    Args:
        new_baud_rate(u64): The new value of the UHF baud rate
     */
    println!("Setting UHF baud rate to: {:?}", new_baud_rate);
}

fn get_hk_data(interface: &mut TcpInterface, content: Vec<u8>) {
    println!("Grabbing house keeping data...");
}

fn reset_uhf(interface: &mut TcpInterface) {
    println!("Resetting UHF...");
}

pub fn handle_uhf_cmd(interface: &mut TcpInterface, msg: &Msg) {
    let opcode = opcodes::UHF::from(msg.header.op_code);
    
    match opcode {
        opcodes::UHF::GetHK => {
            get_hk_data(interface, msg.msg_body.clone());
        },
        opcodes::UHF::SetBeacon => {
            set_beacon_value(interface, msg.msg_body.clone());
        },
        opcodes::UHF::GetBeacon => {
            println!("Got opcode 5 to get beacon value");
            get_beacon_value(interface);
        },
        opcodes::UHF::SetBaudRate => {
            println!("Got opcode 6 to set beacon value");
            set_baud_rate(interface, msg.msg_body.clone());
        },
        opcodes::UHF::GetBaudRate => {
            get_baud_rate(interface)
        },
        opcodes::UHF::Reset => {
            reset_uhf(interface)
        },
        _ => println!("Invalid opcode")
    }

}




#[test]
fn test_set_beacon() {
    let mut interface = TcpInterface::new_client("127.0.0.1".to_string(), 1234).unwrap();
    // Set beacon value to "bacon"
    // Tested using simulated uhf
    // TODO: make better test
    set_beacon_value(&mut interface, vec!(0x62, 0x61, 0x63, 0x6F, 0x6E));
    
}