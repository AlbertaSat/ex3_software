use nix::unistd::close;
use nix::Error;
use strum::IntoEnumIterator;
use std::os::fd::AsRawFd;

use interface::ipc::{poll_ipc_server_sockets, IpcClient, IpcServer, IPC_BUFFER_SIZE};
use common::component_ids::ComponentIds;
use common::message_structure::MsgHeader;


fn main() {
    let mut component_streams: Vec<Option<IpcClient>> =
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
                    println!("{} socket created", payload);
                }
                None => {
                    println!("{} not created!", payload);
                }
            },
            None => println!("bad index {}", x),
        };
    }

    // I think this should be a server. as it does not initiate communication.
    let mut cmd_server = match IpcServer::new("cmd_dispatcher".to_string()) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("Couldn't create command server: {}", e);
            return; // Should fix it and retry
        }
    };

    loop {
        let mut servers = vec![&mut cmd_server];
        // Do not care about cmd clients addr since it only recv from a single socket in coms
        // handler.
        let (_bytes, _addr) = match poll_ipc_server_sockets(&mut servers) {
            Ok((bytes, sock)) => (bytes,sock),
            Err(_e) => {
                let _ = close(cmd_server.as_ref().unwrap().fd.as_raw_fd());
                continue; // try again
            }
        };
        if cmd_server.as_ref().unwrap().buffer != [0u8; IPC_BUFFER_SIZE] {
            // got message, check destination id
            let dest = cmd_server.as_ref().unwrap().buffer[MsgHeader::DEST_INDEX];
            let res = match ComponentIds::try_from(dest) {
                Ok(payload) => {
                    match &mut component_streams[dest as usize] {
                        Some(client) => {
                            let _res = client.send(&cmd_server.as_ref().unwrap().buffer);
                            Ok(())
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
                println!("{:?}", res);
                // Should actually NACK
            }
            cmd_server.as_mut().unwrap().clear_buffer();
        }
    }
}
