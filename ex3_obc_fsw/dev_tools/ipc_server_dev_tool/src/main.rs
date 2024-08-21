/*
Written by Devin Headrick
Summer 2024

Create an IPC server on the path specified as an arg - and send hardcoded data based on user input to stdin.

This is useful to 'plug into' a handler on the Ipc Interface to test it directly.
*/

use common::{
    component_ids::ComponentIds,
    opcodes,
};
use ipc::{ipc_write, poll_ipc_server_sockets, IpcServer, IPC_BUFFER_SIZE};
use message_structure::{serialize_msg, Msg, MsgType};

use nix::poll::{poll, PollFd, PollFlags};
use std::io::{self, Read};

const STDIN_POLL_TIMEOUT_MS: i32 = 100;

fn handle_user_input(component: &str, message_id: &str, ipc_server: &mut IpcServer) {
    println!("Component: {}, Message ID: {}", component, message_id);

    let msg = match component {
        "EPS" => {
            match message_id {
                "1" => {
                    println!("Sending Msg: Ping EPS");
                    let msg = Msg::new(
                        MsgType::Cmd as u8,
                        1,
                        ComponentIds::EPS.into(),
                        ComponentIds::GS.into(),
                        opcodes::EPS::Ping.into(),
                        vec![],
                    );
                    Some(msg)
                }
                //...........
                _ => {
                    println!("Invalid input: Message ID not recognized");
                    None
                }
            }
        }
        // Add more cases here as needed for different components
        _ => {
            println!("Invalid input: Component not recognized");
            None
        }
    };
    if let Some(msg) = msg {
        let serialized_msg = serialize_msg(&msg).unwrap();
        let write_res = ipc_write(ipc_server.data_fd, &serialized_msg.as_slice());
        match write_res {
            Ok(num_bytes_written) => {
                println!("{:?} bytes written to ipc client", num_bytes_written);
            }
            Err(e) => {
                println!("Error Writing to IPC client: {:?}", e);
            }
        }
    }
}

fn poll_stdin(mut buffer: &mut [u8]) -> Option<usize> {
    let stdin_fd = 0; // File descriptor for stdin is always 0
    let mut poll_fds = [PollFd::new(stdin_fd, PollFlags::POLLIN)];

    match poll(&mut poll_fds, STDIN_POLL_TIMEOUT_MS) {
        Ok(1) if poll_fds[0].revents().unwrap().contains(PollFlags::POLLIN) => {
            match io::stdin().read(&mut buffer) {
                Ok(bytes_read) => Some(bytes_read),
                Err(_) => None,
            }
        }
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run --bin ipc_server_dev_tool <path to ipc socket>");
        std::process::exit(1);
    }

    let mut ipc_server = IpcServer::new(args[1].clone()).unwrap();

    loop {
        poll_ipc_server_sockets(&mut vec![&mut ipc_server]);
        if ipc_server.buffer != [0u8; IPC_BUFFER_SIZE] {
            println!("Received message from IPC client {:?}", ipc_server.buffer);
            ipc_server.clear_buffer();
        }

        // Poll stdin for user input
        let mut stdin_buf = [0u8; 1024]; // Adjust buffer size as needed
        let stdin_read_res = poll_stdin(&mut stdin_buf);
        match stdin_read_res {
            Some(bytes_read) => {
                if bytes_read > 0 {
                    let input = String::from_utf8_lossy(&stdin_buf[..bytes_read]);
                    let mut parts = input.trim().split_whitespace();
                    if let (Some(component), Some(message_id)) = (parts.next(), parts.next()) {
                        handle_user_input(component, message_id, &mut ipc_server);
                    } else {
                        println!("Invalid input format. Please enter '<component> <message_id>'.");
                    }
                }
            }
            None => (),
        }
    }
}
