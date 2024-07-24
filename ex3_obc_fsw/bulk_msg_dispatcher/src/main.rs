use ipc::*;
use common::*;
use message_structure::*;

fn main() -> Result<(), std::io::Error> {
    // All connected handlers and other clients will have a socket for the server defined here
    let mut dfgm_interface = IpcServer::new("dfgm_bulk".to_string())?;
    let mut gs_interface = IpcServer::new("gs_bulk".to_string())?;

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
        //TMP - test read/write to dfgm
        // ipc_write(server.data_fd, server.buffer.as_slice())?;
        server.clear_buffer();
    }
    Ok(())
}
