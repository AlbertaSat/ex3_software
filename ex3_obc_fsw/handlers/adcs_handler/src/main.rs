/*
Written by Amar!!
*/
use common::{opcodes, ports};
use ipc_interface::read_socket;
use ipc_interface::IPCInterface;
use message_structure::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use tcp_interface::*;

const ADCS_DATA_DIR_PATH: &str = "adcs_data";
const ADCS_PACKET_SIZE: usize = 1252;
const ADCS_INTERFACE_BUFFER_SIZE: usize = ADCS_PACKET_SIZE;

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
                "Error creating DFGM interface: {:?}",
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
                    Ok(())
                }
                1 => {
                    self.toggle_adcs = true;
                    Ok(())
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
