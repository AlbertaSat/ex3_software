use ipc_interface::*;
use ipc::*;
use common::*;
use message_structure::*;

fn main() -> Result<(), std::io::Error> {
    // All connected handlers and other clients will have a socket for the server defined here
    let mut dfgm_interface = IpcServer::new("dfgm_bulk".to_string())?;


    loop {
        let mut servers: Vec<&mut IpcServer> = vec![&mut dfgm_interface];
        poll_ipc_server_sockets(servers);
        for server in servers {
            handle_client(server);
        }
    }
}


fn handle_client(server: &mut IpcServer) -> Result<(), std::io::Error> {

    Ok(())
}