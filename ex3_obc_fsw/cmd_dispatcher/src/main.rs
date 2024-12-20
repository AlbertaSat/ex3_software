use nix::unistd::{write, close};
use nix::Error;
use strum::IntoEnumIterator;
use std::os::fd::{AsFd, AsRawFd};

use interface::ipc::{poll_ipc_clients, IpcClient, IPC_BUFFER_SIZE};
use common::component_ids::ComponentIds;
use common::message_structure::MsgHeader;

fn main() {
    let component_streams: Vec<Option<IpcClient>> =
        ComponentIds::iter().enumerate().map(|(i,c)| {
            println!("{i}");
            match IpcClient::new(format!("{c}")) {
                Ok(client) => {
                    Some(client)
                }
                Err(e) => {
                    eprintln!("msg dispatcher couldn't connect to {}: {}", c, e);
                    None
                },
            }
        }).collect();
    
    for x in 0..ComponentIds::LAST as usize {
        let payload = match ComponentIds::try_from(x as u8) {
            Ok(p) => {
                eprintln!("x {} yields {}", x, p);
                p
            },
            Err(()) => {
                eprintln!("x {} didn't convert", x);
                continue;
            }
        };
        match component_streams.get(x) {
            Some(element) => match element {
                Some(_e) => {
                    println!("{} connected", payload);
                }
                None => {
                    println!("{} not connected!", payload);
                }
            },
            None => println!("bad index {}", x),
        };
    }

    let mut cmd_client = match IpcClient::new("cmd_dispatcher".to_string()) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("Server connection error: {}", e);
            return; // Should fix it and retry
        }
    };

    loop {
        let mut clients = vec![&mut cmd_client];
        let (_s,_bytes) = match poll_ipc_clients(&mut clients) {
            Ok((bytes, sock)) => (bytes,sock),
            Err(e) => {
                eprintln!("read error: {}", e);
                let _ = close(cmd_client.as_ref().unwrap().fd.as_raw_fd());
                continue; // try again
            }
        };
        if cmd_client.as_ref().unwrap().buffer != [0u8; IPC_BUFFER_SIZE] {
            let dest = cmd_client.as_ref().unwrap().buffer[MsgHeader::DEST_INDEX];
            let res = match ComponentIds::try_from(dest) {
                Ok(payload) => {
                    match &component_streams[dest as usize] {
                        Some(client) => {
                            write(client.fd.as_fd(), &cmd_client.as_ref().unwrap().buffer)
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

            if res.is_err() {
                eprintln!("Dispatch failed: NACKing");
                // Should actually NACK
            }
            cmd_client.as_mut().unwrap().clear_buffer();
        }
    }
}
