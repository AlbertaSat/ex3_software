/*
Written by Amar
*/
use common::{opcodes, ports};
use ipc_interface::read_socket;
use ipc_interface::IPCInterface;
use message_structure::*;
use std::collections::HashMap;
use std::fs::OpenOptions;

use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use tcp_interface::*;

const CMD_DELIMITER: u8 = b":"[0];
const ADCS_DATA_DIR_PATH: &str = "adcs_data";
const ADCS_PACKET_SIZE: usize = 1252;
const ADCS_INTERFACE_BUFFER_SIZE: usize = ADCS_PACKET_SIZE;

// TODO check where to add this
// Probably will move this to another file later
pub mod adcs_body {
    struct ADCSCmdParam<'a>(&'a [u8], i32);
    pub const ON: ADCSCmdParam = ADCSCmdParam(b"ON", 0);
    pub const OFF: ADCSCmdParam = ADCSCmdParam(b"OFF", 0);
    pub const GET_STATE: ADCSCmdParam = ADCSCmdParam(b"GS", 0);
    pub const GET_WHEEL_SPEED: ADCSCmdParam = ADCSCmdParam(b"GWS", 0);
    pub const SET_WHEEL_SPEED: ADCSCmdParam = ADCSCmdParam(b"SWS", 3);
}

struct ADCSHandler {
    toggle_adcs: bool, // TODO make this more related to booting-up possibly? (affects sim sub sys as well)
    peripheral_interface: Option<TcpInterface>,
    dispatcher_interface: Option<IPCInterface>,
}

impl ADCSHandler {
    pub fn new(
        adcs_interface: Result<TcpInterface, std::io::Error>,
        dispatcher_interface: Result<IPCInterface, std::io::Error>,
    ) -> ADCSHandler {
        if adcs_interface.is_err() {
            println!(
                "Error creating ADCS interface: {:?}",
                adcs_interface.as_ref().err().unwrap()
            );
        }
        if dispatcher_interface.is_err() {
            println!(
                "Error creating dispatcher interface: {:?}",
                dispatcher_interface.as_ref().err().unwrap()
            );
        }

        ADCSHandler {
            toggle_adcs: false,
            peripheral_interface: adcs_interface.ok(),
            dispatcher_interface: dispatcher_interface.ok(),
        }
    }

    fn handle_msg_for_adcs(&mut self, msg: Msg) -> Result<(), Error> {
        match msg.header.op_code {
            opcodes::adcs::DETUMBLE => {
                eprintln!("Error: Detumble is not implemented");
                Err(Error::new(
                    ErrorKind::NotFound,
                    "Detumble is not implemented for the ADCS yet",
                ))
            }
            opcodes::adcs::ON_OFF => match msg.msg_body[0] {
                0 => {
                    self.toggle_adcs = false;
                    self.send_cmd(adcs_body::OFF)
                }
                1 => {
                    self.toggle_adcs = true;
                    self.send_cmd(adcs_body::ON)
                }
                2 => {
                    self.toggle_adcs = true;
                    self.send_cmd(adcs_body::GET_STATE)
                }
                _ => {
                    eprintln!("Error: Unknown msg body for opcode 2");
                    Err(Error::new(
                        ErrorKind::NotFound,
                        "Error: Unknown msg body for opcode 2",
                    ))
                }
            },
            _ => {
                eprintln!(
                    "{}",
                    format!("Opcode {} not found for ADCS", msg.header.op_code)
                );
                Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Opcode {} not found for ADCS", msg.header.op_code),
                ))
            }
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let mut socket_buf = vec![0u8; ADCS_INTERFACE_BUFFER_SIZE];
        loop {
            if let Ok(n) = read_socket(
                self.dispatcher_interface.clone().unwrap().fd,
                &mut socket_buf,
            ) {
                if n > 0 {
                    let recv_msg: Msg = deserialize_msg(&socket_buf).unwrap();
                    self.handle_msg_for_adcs(recv_msg)?;
                    println!("Data toggle set to {}", self.toggle_adcs);
                    socket_buf.flush()?;
                }
            }
        }
    }

    fn build_cmd(&mut self, cmd: &[u8], msg: Msg) -> Result<Vec<u8>, ()> {
        let mut data: Vec<u8>;
        if let Some(num_params) = ADCS_PARAMS.get(cmd) {
            if msg.msg_body.len() < (1 + *num_params).try_into().unwrap() {
                return Err(());
            }
            for val in msg.msg_body {
                data.push(CMD_DELIMITER);
                data.push(val);
            }
            Ok(data)
        } else {
            Err(())
        }
    }

    fn send_cmd(&mut self, command: &[u8]) -> Result<(), Error> {
        self.peripheral_interface.as_mut().unwrap().send(command)?;
        Ok(())
    }
}

fn main() {
    println!("Beginning ADCS Handler...");
    //For now interfaces are created and if their associated ports are not open, they will be ignored rather than causing the program to panic

    //Create TCP interface for ADCS handler to talk to simulated ADCS
    let adcs_interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_ADCS_PORT);

    //Create TCP interface for ADCS handler to talk to message dispatcher
    let dispatcher_interface = IPCInterface::new("adcs_handler".to_string());

    //Create ADCS handler
    let mut adcs_handler = ADCSHandler::new(adcs_interface, dispatcher_interface);

    adcs_handler.run();
}
