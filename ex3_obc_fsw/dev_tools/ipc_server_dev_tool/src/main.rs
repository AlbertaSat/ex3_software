/*
Written by Devin Headrick
Summer 2024

Create an ipc server on the path specified as an arg - and send hardcoded data based on user input to stdin

*/

use common::component_ids::{ComponentIds};
use ipc::{ipc_write, poll_ipc_server_sockets, IpcServer, IPC_BUFFER_SIZE};
use message_structure::{CmdMsg, SerializeAndDeserialize};

use nix::poll::{poll, PollFd, PollFlags};
use std::io::{self, Read};

const STDIN_POLL_TIMEOUT_MS: i32 = 100;

/// Write a messaage to the IPC - user enteres number to send assoicated example message
fn handle_user_input(read_data: &[u8], ipc_server: &mut IpcServer) {
    let first_byte = read_data[0];
    let first_byte_char = first_byte as char;
    println!("First byte: {}", first_byte_char);

    let rc = match first_byte_char {
        '1' => {
            println!("Sending msg 1");
            //write first hardcoded msg to ipc client
            let msg = CmdMsg::new(1, ComponentIds::DUMMY.into(), 3, 0, vec![5, 6, 7, 8, 9, 10]);
            let serialized_msg = CmdMsg::serialize_to_bytes(&msg).unwrap();
            ipc_write(ipc_server.data_fd, &serialized_msg.as_slice())
        }
        '2' => {
            println!("Sending msg 2");
            //write first hardcoded msg to ipc client
            let msg = CmdMsg::new(2, ComponentIds::DUMMY.into(), 3, 1, vec![5, 6, 7, 8, 9, 10]);
            let serialized_msg = CmdMsg::serialize_to_bytes(&msg).unwrap();
            ipc_write(ipc_server.data_fd, &serialized_msg.as_slice())
        }
        _ => {
            println!("Invalid input");
            Ok(0)
        }
    };
    if let Some(e) = rc.err() {
        println!("ipc_write error: {e}");
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
            println!("Received message from ipc client {:?}", ipc_server.buffer);
            ipc_server.clear_buffer();
        }

        // Poll stdin for user input
        let mut stdin_buf = [0u8; 1024]; // Adjust buffer size as needed
        let stdin_read_res = poll_stdin(&mut stdin_buf);
        match stdin_read_res {
            Some(bytes_read) => {
                if bytes_read > 0 {
                    println!("Received user input: {:?}", stdin_buf);
                    handle_user_input(stdin_buf.as_slice(), &mut ipc_server);
                }
            }
            None => (),
        }
    }
}
