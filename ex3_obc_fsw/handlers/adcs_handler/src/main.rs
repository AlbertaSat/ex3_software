/*
Written by Amar
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

const CMD_DELIMITER: u8 = b":"[0];
const ADCS_DATA_DIR_PATH: &str = "adcs_data";
const ADCS_PACKET_SIZE: usize = 1252;
const ADCS_INTERFACE_BUFFER_SIZE: usize = ADCS_PACKET_SIZE;

// TODO check where to add this
// Probably will move this to another file later
pub mod adcs_body {
    pub struct ADCSCmdParam<'a> {
        pub data: &'a [u8],
        pub params: i32,
    }
    pub const ON: ADCSCmdParam = ADCSCmdParam {
        data: b"ON",
        params: 0,
    };
    pub const OFF: ADCSCmdParam = ADCSCmdParam {
        data: b"OFF",
        params: 0,
    };
    pub const GET_STATE: ADCSCmdParam = ADCSCmdParam {
        data: b"GS",
        params: 0,
    };
    pub const GET_WHEEL_SPEED: ADCSCmdParam = ADCSCmdParam {
        data: b"GWS",
        params: 0,
    };
    pub const SET_WHEEL_SPEED: ADCSCmdParam = ADCSCmdParam {
        data: b"SWS",
        params: 3,
    };
    pub const STATUS_CHECK: ADCSCmdParam = ADCSCmdParam {
        data: b"SC",
        params: 0,
    };
    pub const SET_MAGNETORQUER_CURRENT: ADCSCmdParam = ADCSCmdParam {
        data: b"SMC",
        params: 3,
    };
    pub const GET_MAGNETORQUER_CURRENT: ADCSCmdParam = ADCSCmdParam {
        data: b"GMC",
        params: 0,
    };
    pub const GET_TIME: ADCSCmdParam = ADCSCmdParam {
        data: b"GTM",
        params: 0,
    };
    pub const SET_TIME: ADCSCmdParam = ADCSCmdParam {
        data: b"STM",
        params: 1,
    };
    pub const GET_ORIENTATION: ADCSCmdParam = ADCSCmdParam {
        data: b"GOR",
        params: 0,
    };
    pub const RESET: ADCSCmdParam = ADCSCmdParam {
        data: b"RESET",
        params: 0,
    };
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
                    self.send_cmd(adcs_body::OFF, msg)
                }
                1 => {
                    self.toggle_adcs = true;
                    self.send_cmd(adcs_body::ON, msg)
                }
                2 => self.send_cmd(adcs_body::GET_STATE, msg),
                _ => Err(self.invalid_msg_body(msg)),
            },
            opcodes::adcs::WHEEL_SPEED => match msg.msg_body[0] {
                0 => self.send_cmd(adcs_body::GET_WHEEL_SPEED, msg),
                1 => self.send_cmd(adcs_body::SET_WHEEL_SPEED, msg),
                _ => Err(self.invalid_msg_body(msg)),
            },
            opcodes::adcs::GET_HK => self.send_cmd(adcs_body::STATUS_CHECK, msg),
            opcodes::adcs::MAGNETORQUER_CURRENT => match msg.msg_body[0] {
                0 => self.send_cmd(adcs_body::GET_MAGNETORQUER_CURRENT, msg),
                1 => self.send_cmd(adcs_body::SET_MAGNETORQUER_CURRENT, msg),
                _ => Err(self.invalid_msg_body(msg)),
            },
            opcodes::adcs::ONBOARD_TIME => match msg.msg_body[0] {
                0 => self.send_cmd(adcs_body::GET_TIME, msg),
                1 => self.send_cmd(adcs_body::SET_TIME, msg),
                _ => Err(self.invalid_msg_body(msg)),
            },
            opcodes::adcs::GET_ORIENTATION => self.send_cmd(adcs_body::GET_ORIENTATION, msg),
            opcodes::adcs::RESET => self.send_cmd(adcs_body::RESET, msg),
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

            if self.toggle_adcs == true {
                let mut tcp_buf = [0u8; BUFFER_SIZE];
                let status = TcpInterface::read(
                    &mut self.peripheral_interface.as_mut().unwrap(),
                    &mut tcp_buf,
                );
                match status {
                    Ok(data_len) => {
                        // Notably, the TCP interface will send all 0's when there is no data to send
                        let mut all_zero = true;
                        for i in 0..BUFFER_SIZE {
                            if tcp_buf[i] != 0 {
                                all_zero = false;
                            }
                        }

                        if !all_zero {
                            println!("Got data {:?}", tcp_buf);
                            store_adcs_data(&tcp_buf)?;
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }

    fn build_cmd(&mut self, cmd: adcs_body::ADCSCmdParam, msg: Msg) -> Result<Vec<u8>, Error> {
        let mut data: Vec<u8> = vec![];
        data.extend_from_slice(cmd.data);

        // TODO: later figure out how to check we've sent the correct amount of parameters using an end-body flag maybe use 0xFF?
        // First param in msg body will specify the operation type e.g. getting or setting
        for i in 1..((cmd.params + 1) as usize) {
            data.push(CMD_DELIMITER);
            data.extend_from_slice(msg.msg_body[i].to_string().as_bytes());
        }

        Ok(data)
    }

    fn send_cmd(&mut self, command: adcs_body::ADCSCmdParam, msg: Msg) -> Result<(), Error> {
        let cmd = self.build_cmd(command, msg)?;
        self.peripheral_interface.as_mut().unwrap().send(&cmd)?;

        Ok(())
    }

    fn invalid_msg_body(&mut self, msg: Msg) -> Error {
        eprintln!("Error: Unknown msg body for opcode {}", msg.header.op_code);
        Error::new(
            ErrorKind::NotFound,
            format!("Error: Unknown msg body for opcode {}", msg.header.op_code),
        )
    }
}

fn store_adcs_data(data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(ADCS_DATA_DIR_PATH)?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/data", ADCS_DATA_DIR_PATH))?;
    file.write_all(data)?;
    Ok(())
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
