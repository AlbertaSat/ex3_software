/*
Written by Amar Kulovac

The ADCS subsystem controls the attitude of the satellite, currently the handler is
based around the simulated subsystem which the commands for it can be found in
ex3_simulated_subsystems/ADCS/

TODO: figure out how to cleanly handle errors such as improper inputs
TODO: get an idea of the actual ADCS commands and figure out a clean way to send commands
*/
use common::{opcodes, ports};
use interface::{ipc::*, tcp::*, Interface};
use log::{debug, trace, warn};
use common::logging::*;
use common::message_structure::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;

const CMD_DELIMITER: u8 = b":"[0];
const ADCS_DATA_DIR_PATH: &str = "ex3_obc_fsw/handlers/adcs_handler/adcs_data";
const ADCS_PACKET_SIZE: usize = 1024;

// TODO check if there is a cleaner way to do this
/// This represents the simulated subsystems expected commands
pub mod sim_adcs {
    pub struct ADCSCmdParam<'a> {
        pub data: &'a [u8],
        pub params: usize,
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
    peripheral_interface: Option<TcpInterface>,
    dispatcher_interface: Option<IpcClient>,
}

impl ADCSHandler {
    pub fn new(
        adcs_interface: Result<TcpInterface, std::io::Error>,
        dispatcher_interface: Result<IpcClient, std::io::Error>,
    ) -> ADCSHandler {
        if adcs_interface.is_err() {
            warn!(
                "Error creating ADCS interface: {:?}",
                adcs_interface.as_ref().err().unwrap()
            );
        }
        if dispatcher_interface.is_err() {
            warn!(
                "Error creating dispatcher interface: {:?}",
                dispatcher_interface.as_ref().err().unwrap()
            );
        }

        ADCSHandler {
            peripheral_interface: adcs_interface.ok(),
            dispatcher_interface: dispatcher_interface.ok(),
        }
    }

    fn handle_msg_for_adcs(&mut self, msg: Msg) -> Result<(), Error> {
        match opcodes::ADCS::from(msg.header.op_code) {
            opcodes::ADCS::Detumble => {
                warn!("Error: Detumble is not implemented");
                Err(Error::new(
                    ErrorKind::NotFound,
                    "Detumble is not implemented for the ADCS yet",
                ))
            }

            opcodes::ADCS::OnOff => match msg.msg_body[0] {
                0 => self.send_cmd(sim_adcs::OFF, msg),
                1 => self.send_cmd(sim_adcs::ON, msg),
                2 => self.send_cmd(sim_adcs::GET_STATE, msg),
                _ => Err(self.invalid_msg_body(msg)),
            },

            opcodes::ADCS::WheelSpeed => match msg.msg_body[0] {
                0 => self.send_cmd(sim_adcs::GET_WHEEL_SPEED, msg),
                1 => self.send_cmd(sim_adcs::SET_WHEEL_SPEED, msg),
                _ => Err(self.invalid_msg_body(msg)),
            },

            opcodes::ADCS::GetHk => self.send_cmd(sim_adcs::STATUS_CHECK, msg),

            opcodes::ADCS::MagnetorquerCurrent => match msg.msg_body[0] {
                0 => self.send_cmd(sim_adcs::GET_MAGNETORQUER_CURRENT, msg),
                1 => self.send_cmd(sim_adcs::SET_MAGNETORQUER_CURRENT, msg),
                _ => Err(self.invalid_msg_body(msg)),
            },

            opcodes::ADCS::OnboardTime => match msg.msg_body[0] {
                0 => self.send_cmd(sim_adcs::GET_TIME, msg),
                1 => self.send_cmd(sim_adcs::SET_TIME, msg),
                _ => Err(self.invalid_msg_body(msg)),
            },

            opcodes::ADCS::GetOrientation => self.send_cmd(sim_adcs::GET_ORIENTATION, msg),

            opcodes::ADCS::Reset => self.send_cmd(sim_adcs::RESET, msg),

            _ => {
                warn!(
                    "{}",
                    format!("Error: Opcode {} not found for ADCS", msg.header.op_code)
                );
                Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Error: Opcode {} not found for ADCS", msg.header.op_code),
                ))
            }
        }
    }

    /// Main loop for ADCS Handler
    pub fn run(&mut self) -> std::io::Result<()> {
        loop {
            if let Ok((n, _)) =
                poll_ipc_clients(&mut vec![&mut self.dispatcher_interface])
            {
                if n > 0 {
                    let mut socket_buf = self.dispatcher_interface.as_mut().unwrap().read_buffer();

                    self.handle_dispatcher_msg(&mut socket_buf)?;
                    self.handle_data_storing()?;
                }
            }
        }
    }

    /// Takes the bytes read from the IPC interface and
    /// sends it to the ADCS if an error occurred the msg
    /// is stored in ADCS data
    fn handle_dispatcher_msg(&mut self, buf: &mut Vec<u8>) -> std::io::Result<()> {
        let recv_msg: Msg = deserialize_msg(buf).unwrap();

        if let Err(invalid_cmd) = self.handle_msg_for_adcs(recv_msg) {
            // TODO: create some meaningful error handling here
            let mut msg: Vec<u8> = vec![];

            msg.extend_from_slice(invalid_cmd.to_string().as_bytes());
            pad_zeros(&mut msg, ADCS_PACKET_SIZE)?;

            store_adcs_data(&msg)?;
        }

        buf.flush()
    }

    /// Reads from the tcp buffer and stores non-zero messages
    /// in ADCS Data
    fn handle_data_storing(&mut self) -> Result<(), Error> {
        let mut tcp_buf = [0u8; BUFFER_SIZE];
        let status = TcpInterface::read(self.peripheral_interface.as_mut().unwrap(), &mut tcp_buf);

        match status {
            Ok(_) => {
                // Notably, the TCP interface will send all 0's when there is no data to send
                if tcp_buf != [0u8; ADCS_PACKET_SIZE] {
                    debug!("Got data {:?}", tcp_buf);

                    // converts the tcp_buf into a `String`
                    let adcs_msg: String = tcp_buf
                        .iter()
                        .take_while(|&&x| x != 0)
                        .map(|&x| x as char)
                        .collect();
                    trace!("ADCS MSG: \"{}\"", adcs_msg);

                    store_adcs_data(&tcp_buf)?;
                }
            }
            Err(e) => {
                warn!("Error: {}", e);
            }
        }

        Ok(())
    }

    /// Builds commands to follow the simulated subsystems expected command structure
    fn build_cmd(&mut self, cmd: sim_adcs::ADCSCmdParam, msg: Msg) -> Result<Vec<u8>, Error> {
        let mut data: Vec<u8> = vec![];
        data.extend_from_slice(cmd.data);

        // TODO: later figure out how to check we've sent the correct amount of parameters using an end-body flag maybe use 0xFF?
        // First param in msg body will specify the operation type e.g. getting or setting
        for i in 1..(cmd.params + 1) {
            data.push(CMD_DELIMITER);
            data.extend_from_slice(msg.msg_body[i].to_string().as_bytes());
        }

        Ok(data)
    }

    fn send_cmd(&mut self, command: sim_adcs::ADCSCmdParam, msg: Msg) -> Result<(), Error> {
        let cmd = self.build_cmd(command, msg)?;
        self.peripheral_interface.as_mut().unwrap().send(&cmd)?;

        Ok(())
    }

    fn invalid_msg_body(&mut self, msg: Msg) -> Error {
        warn!(
            "Error: Unknown msg body for opcode {}, {}",
            msg.header.op_code,
            opcodes::ADCS::from(msg.header.op_code)
        );
        Error::new(
            ErrorKind::NotFound,
            format!(
                "Error: Unknown msg body for opcode {}, {}",
                msg.header.op_code,
                opcodes::ADCS::from(msg.header.op_code)
            ),
        )
    }
}

/// Helper function to pad an array to a length "n"
fn pad_zeros(array: &mut Vec<u8>, n: usize) -> std::io::Result<()> {
    for _ in 0..(n - array.len()) {
        array.push(0);
    }

    Ok(())
}

/// Stores `data` into `adcs_data/data`
fn store_adcs_data(data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(ADCS_DATA_DIR_PATH)?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/data", ADCS_DATA_DIR_PATH))?;
    file.write_all(data)?;
    Ok(())
}

fn main() -> Result<(), Error> {
    //For now interfaces are created and if their associated ports are not open, they will be ignored rather than causing the program to panic

    let log_path = "ex3_obc_fsw/handlers/adcs_handler/logs";
    init_logger(log_path);
    trace!("Logger initialized");
    trace!("Beginning ADCS Handler...");

    //Create TCP interface for ADCS handler to talk to simulated ADCS
    let adcs_interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_ADCS_PORT);

    //Create IPC interface for ADCS handler to talk to message dispatcher
    let dispatcher_interface = IpcClient::new("ADCS".to_string());

    //Create ADCS handler
    let mut adcs_handler = ADCSHandler::new(adcs_interface, dispatcher_interface);

    adcs_handler.run()
}
