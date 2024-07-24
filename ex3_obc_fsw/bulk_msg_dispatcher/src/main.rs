use ipc::*;
use std::fs::{OpenOptions, File};
use std::io::Read;
use common::*;
use message_structure::*;

fn main() -> Result<(), std::io::Error> {
    // All connected handlers and other clients will have a socket for the server defined here
    let mut dfgm_interface: IpcServer = IpcServer::new("dfgm_bulk".to_string())?;
    let mut gs_interface: IpcServer = IpcServer::new("gs_bulk".to_string())?;

    let mut servers: Vec<&mut IpcServer> = vec![&mut dfgm_interface, &mut gs_interface];    
    loop {
        poll_ipc_server_sockets(&mut servers);
        
        for server in &mut servers {
            handle_client(server)?;
        }
    }
}

fn handle_client(server: &mut IpcServer) -> Result<(), std::io::Error> {
    if server.buffer != [0u8; IPC_BUFFER_SIZE] {
        println!(
            "Server {} received data: {:?}",
            server.socket_path,
            &server.buffer
        );
        //Build Msg from received bytes and get body which contains path
        let msg: Msg = deserialize_msg(&server.buffer)?;
        let path_bytes: Vec<u8> = msg.msg_body;
        let path = String::from_utf8(path_bytes).expect("Found invalid UTF-8 in path.");
        path = path.trim_matches(char::from(0));
        println!("Got path: {}", path);
        let bulk_msg: Msg = get_data_from_path(&path)?;
        // Start communication protocol for Bulk Msg with GS handler

        server.clear_buffer();
    }
    Ok(())
}

/// This function will take all current data that is stored in a provided path and
/// append it to the body of a bulk Msg. This Msg will then be sliced.
fn get_data_from_path(path: &str) -> Result<Msg, std::io::Error> {
    let mut file: File = OpenOptions::new()
        .read(true)
        .open(format!("{}/data", path))?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;
    let bulk_msg: Msg = Msg::new(2,0,7,3,0,data);
    Ok(bulk_msg)
}
