use nix::sys::socket::accept;
use nix::unistd::{read, write, close};
use nix::Error;
use strum::IntoEnumIterator;
use std::os::fd::{AsFd, AsRawFd};

use ipc::{IpcClient, IpcServer, IPC_BUFFER_SIZE};
use common::component_ids::ComponentIds;
use message_structure::MsgHeaderNew;

fn main() {
    let component_streams: Vec<Option<IpcClient>> =
        ComponentIds::iter().map(|c| {
            match IpcClient::new(format!("{c}")) {
                Ok(client) => Some(client),
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
                Some(_) => println!("{} connected", payload),
                None => println!("{} not connected!", payload),
            },
            None => println!("bad index {}", x),
        };
    }

    let server = match IpcServer::new("msg_dispatcher".to_string()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Server connection error: {}", e);
            return; // Should fix it and retry
        }
    };

    let data_fd = match accept(server.conn_fd.as_raw_fd()) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("accept failed: {}", e);
            -1
        }
    };

    loop {

        let mut buffer = [0; IPC_BUFFER_SIZE];
        let _bytes_read = match read(data_fd, &mut buffer) {
            Ok(len) => len,
            Err(e) => {
                eprintln!("read error: {}", e);
                let _ = close(data_fd);
                continue; // try again
            }
        };

        let dest = buffer[MsgHeaderNew::DEST_INDEX];
        let res = match ComponentIds::try_from(dest) {
            Ok(payload) => {
                match &component_streams[dest as usize] {
                    Some(client) => {
                        write(client.fd.as_fd(), &buffer)
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
        let _= close(data_fd);
    }
}
