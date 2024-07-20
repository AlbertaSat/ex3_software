use ipc_interface::*;
use common::*;
use message_structure::*;

fn main() -> Result<(), std::io::Error> {
    let dfgm_interface = IPCServerInterface::new_server("dfgm_bulk".to_string())?;
    let gs_interface = IPCServerInterface::new_server("gs_bulk".to_string())?;
    let mut connected_clients: Vec<IPCServerInterface> = vec![dfgm_interface, gs_interface.clone()];
    let socket_buf: &mut Vec<u8> = &mut vec![0u8; IPC_BUFFER_SIZE];
    let arb_data = Msg::new(0,0,0,0,0,"<path>".as_bytes().to_vec());
    loop {
        poll_server_interfaces(&mut connected_clients, socket_buf);
    }
}


fn handle_client(client_fd: i32) -> Result<(), std::io::Error> {
    let mut ipc_initial_buf = vec![0u8; 128];
    match read_socket(client_fd, &mut ipc_initial_buf) {
        Ok(num_bytes_read) => {
            if num_bytes_read > 0 {
                println!("Received {} IPC Msg bytes", num_bytes_read);
                let deserialized_msg = deserialize_msg(&ipc_initial_buf.as_slice())?;
                // TMP - testing sending and clients ability to read
                send_over_socket(client_fd, serialize_msg(&deserialized_msg)?)?;
                println!("Sent {:?}", deserialized_msg);
                // match deserialized_msg_result {
                        //     Ok(deserialized_msg) => {
                        //         // Fetch data from path in body. Get string from bytes in body
                        //         todo!()
                        //     }
                        //     Err(e) => {
                        //         println!("Error deserializing GS IPC msg: {}", e);
                        //     }
                        // };
            }
        }
        Err(e) => return Err(e),
    }
    Ok(())
}