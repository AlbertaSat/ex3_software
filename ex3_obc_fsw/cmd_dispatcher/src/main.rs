use nix::sys::socket::accept;
use nix::unistd::{read, write, close};
use nix::Error;
use strum::IntoEnumIterator;
use std::os::fd::{AsFd, RawFd, AsRawFd};

use ipc::{IpcClient, IpcServer, IPC_BUFFER_SIZE};
use common::component_ids::ComponentIds;
use message_structure::MsgHeaderNew;

fn main() {
    let component_streams: [Option<IpcClient>; ComponentIds::LAST as usize];
    for x in 0..(ComponentIds::LAST as usize) {
        component_streams[x] = {
            match ComponentIds::try_from(x as u8) {
                Ok(c) => {
                    match IpcClient::new(format!("{c}")) {
                        Ok(client) => Some(client),
                        Err(e) => {
                            eprintln!("msg dispatcher couldn't connect to {}: {}", c, e);
                            None
                        },
                    }
                },
                Err(_) => None,
            }
        };
    }

    for x in 0..(ComponentIds::LAST as usize) {
        let payload = ComponentIds::try_from(x as u8).unwrap();
        match component_streams[x] {
            Some(_) => println!("{} connected", payload),
            None => println!("{} not connected!", payload),
        };
    }

    let server = match IpcServer::new("msg_dispatcher".to_string()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Server connection error: {}", e);
            return; // Should fix it and retry
        }
    };
    
    loop {
        let data_fd = match accept(server.conn_fd.as_raw_fd()) {
            Ok(fd) => fd,
            Err(e) => {
                eprintln!("accept failed: {}", e);
                break; // just start over
            }
        };

        let mut buffer = [0; IPC_BUFFER_SIZE];
        let bytes_read = match read(data_fd, &mut buffer) {
            Ok(len) => len,
            Err(e) => {
                eprintln!("read error: {}", e);
                close(data_fd);
                continue; // try again
            }
        };

        let dest = buffer[MsgHeaderNew::DEST_INDEX];
        let res = match ComponentIds::try_from(dest) {
            Ok(payload) => {
                match &component_streams[dest as usize] {
                    Some(client) => {
                        write(client.fd, &buffer)
                    },
                    None => {
                        eprintln!("No payload: {payload}!");
                        Err(Error::EPIPE)
                    }
                }
            },
            Err(_) => {
                eprintln!("Invalid payload: {dest}");
                Err(Error::EINVAL)
            }
        };

        if let Err(_) = res {
            eprintln!("Dispatch failed: NACKing");
            // Should actually NACK
        }
        close(data_fd);
    }
}
