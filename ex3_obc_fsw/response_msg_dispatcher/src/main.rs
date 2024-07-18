use ipc_interface::*;
use common::*;
use message_structure::*;
use std::collections::HashSet;

fn main() -> Result<(), std::io::Error> {
    // Setup communication with handler(s) and coms_handler
    let dfgm_interface = IPCInterface::new_server("bulk_dfgm".to_string())?;
    let mut connected_clients: HashSet<i32> = HashSet::new();


    loop {
        match dfgm_interface.accept_connection() {
            Ok(client_fd) => {
                println!("New client accepted on fd: {}", client_fd);
                connected_clients.insert(client_fd);
            }
            Err(e) => {
                println!("Error accepting connection: {}", e);
            }
        }

        // Poll existing connections
        let mut to_remove = Vec::new();
        for &client_fd in &connected_clients {
            let mut ipc_initial_buf = vec![0u8;128];
            let ipc_bytes_read_result = read_socket(client_fd, &mut ipc_initial_buf);
            match ipc_bytes_read_result {
                Ok(num_bytes_read) => {
                    if num_bytes_read > 0 {
                        println!("Received {} IPC Msg bytes", num_bytes_read);
                        // Deserial Msg to look at body - path to data
                        let deserialized_msg_result = deserialize_msg(&ipc_initial_buf.as_slice());
                        match deserialized_msg_result {
                            Ok(deserialized_msg) => {
                                // Fetch data from path in body. Get string from bytes in body
                                todo!()
                            }
                            Err(e) => {
                                println!("Error deserializing GS IPC msg: {}", e);
                            }
                        };
                    }
                }
                Err(e) => {
                    println!("Error reading from DFGM IPC socket: {:?}", e);
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        println!("Client disconnected: {}", client_fd);
                        to_remove.push(client_fd);
                    }
                }
            }
        }
        for client_fd in to_remove {
            connected_clients.remove(&client_fd);
        }
    }
}
