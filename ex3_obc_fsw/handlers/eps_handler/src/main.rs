/*
Written by Kaaden RumanCam
Fall 2024
*/

use log::{debug, trace, warn};
use std::io::Error;

use common::{logging::*, message_structure::*, opcodes, ports};
use common::component_ids::ComponentIds::{EPS, GS};
use common::constants::DOWNLINK_MSG_BODY_SIZE;
use interface::{ipc::*, tcp::*, Interface};

struct EPSHandler {
    eps_interface: Option<TcpInterface>, // To communicate with the EPS
    msg_dispatcher_interface: Option<IpcServer>, // For communcation with other FSW components [internal to OBC]
    gs_interface: Option<IpcClient>, // To send messages to the GS through the coms_handler
}

impl EPSHandler {
    pub fn new(
        eps_interface: Result<TcpInterface, std::io::Error>,
        msg_dispatcher_interface: Result<IpcServer, std::io::Error>,
        gs_interface: Result<IpcClient, std::io::Error>,
    ) -> EPSHandler {
        if eps_interface.is_err() {
            warn!(
                "Error creating eps interface: {:?}",
                eps_interface.as_ref().err().unwrap()
            );
        }
        if msg_dispatcher_interface.is_err() {
            warn!(
                "Error creating dispatcher interface: {:?}",
                msg_dispatcher_interface.as_ref().err().unwrap()
            );
        }
        if gs_interface.is_err() {
            warn!(
                "Error creating gs interface: {:?}",
                gs_interface.as_ref().err().unwrap()
            );
        }
        EPSHandler {
            eps_interface: eps_interface.ok(),
            msg_dispatcher_interface: msg_dispatcher_interface.ok(),
            gs_interface: gs_interface.ok(),
        }
    }

    // Sets up threads for reading and writing to its interaces, and sets up channels for communication between threads and the handler
    pub fn run(&mut self) -> std::io::Result<()> {
        // Poll for messages
        loop {
            // First, take the Option<IpcClient> out of `self.dispatcher_interface`
            // This consumes the Option, so you can work with the owned IpcClient
            let msg_dispatcher_interface = self.msg_dispatcher_interface.take().expect("Cmd_Disp has value of None");

            // Create a mutable Option<IpcClient> so its lifetime persists
            let mut msg_dispatcher_interface_option = Some(msg_dispatcher_interface);

            // Now you can borrow this mutable option and place it in the vector
            let mut server: Vec<&mut Option<IpcServer>> = vec![
                &mut msg_dispatcher_interface_option,
            ];

            let _ = poll_ipc_server_sockets(&mut server);

            // restore the value back into `self.dispatcher_interface` after polling. May have been mutated
            self.msg_dispatcher_interface = msg_dispatcher_interface_option;

            // Handling the bulk message dispatcher interface
            let msg_dispatcher_interface = self.msg_dispatcher_interface.as_ref().unwrap();
            if msg_dispatcher_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
                let recv_msg: Msg = deserialize_msg(&msg_dispatcher_interface.buffer).unwrap();
                debug!("Received and deserialized msg");
                self.handle_msg(recv_msg)?;
            }
        }
    }

    fn handle_msg(&mut self, msg: Msg) -> Result<(), Error> {
        self.msg_dispatcher_interface.as_mut().unwrap().clear_buffer();

        trace!("EPS msg opcode: {} {:?}", msg.header.op_code, msg.msg_body);

        let mut tcp_buf = [0u8;BUFFER_SIZE];

        let opcode = opcodes::EPS::from(msg.header.op_code);
        let mut cmd = "dummy";
        match opcode {
            opcodes::EPS::On => {
                trace!("on");
            }
            opcodes::EPS::Off => {
                trace!("off");
            }
            opcodes::EPS::GetHK => {
                trace!("gethk");
                cmd = "request:Temperature";
            }
            opcodes::EPS::Reset => {
                trace!("reset");
                cmd = "execute:ResetDevice";
            }
            opcodes::EPS::Error => {
                debug!("Unrecognised opcode");
            }
        }

        TcpInterface::send(self.eps_interface.as_mut().unwrap(), cmd.as_bytes())?;
        TcpInterface::read(self.eps_interface.as_mut().unwrap(), &mut tcp_buf)?;
        let tmp = String::from_utf8(tcp_buf.to_vec()).unwrap();
        let mut resp = tmp.trim_end_matches(char::from(0)).to_string();
        trace!("From EPS got: {:?}",resp);
        resp.truncate(DOWNLINK_MSG_BODY_SIZE);

        let msg = Msg::new(MsgType::Cmd as u8, 0, GS as u8, EPS as u8, 0, resp.as_bytes().to_vec());
        if let Some(ref mut gs_resp_interface) = self.gs_interface {
            let _ = gs_resp_interface.send(&serialize_msg(&msg)?);
        } else  {
            debug!("Response not sent to gs. IPC interface not created");
        }

        Ok(())
    }
}

fn main() {
    let log_path = "ex3_obc_fsw/handlers/eps_handler/logs";
    init_logger(log_path);

    trace!("Starting EPS Handler...");

    // Create Unix domain socket interface for to talk to message dispatcher
    let msg_dispatcher_interface = IpcServer::new("EPS".to_string());

    let gs_interface = IpcClient::new("gs_non_bulk".to_string());

    let eps_interface = TcpInterface::new_client("127.0.0.1".to_string(), ports::SIM_EPS_PORT);

    let mut eps_handler = EPSHandler::new(eps_interface, msg_dispatcher_interface, gs_interface);

    let _ = eps_handler.run();
}
