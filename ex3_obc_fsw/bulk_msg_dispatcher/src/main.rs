use ipc_interface::*;
use common::*;
use message_structure::*;

fn main() -> Result<(), std::io::Error> {
    // Setup communication with handler(s) and coms_handler
    let dfgm_interface = IPCInterface::new_server("bulk_dfgm".to_string())?;
    let mut dfgm_ipc_initial_buf = Vec::with_capacity(128);
    let mut dfgm_ipc_num_bytes_read: usize = 0;


    loop{
        // Poll both the UHF transceiver and IPC unix domain socket for the GS channel
        let dfgm_ipc_bytes_read_result = read_socket(dfgm_interface.fd, &mut dfgm_ipc_initial_buf);
        match dfgm_ipc_bytes_read_result {
            Ok(num_bytes_read) => {
                dfgm_ipc_num_bytes_read = num_bytes_read;

            }
            Err(e) => {
                println!("Error reading from DFGM IPC socket: {:?}", e);
            }
        }

        if dfgm_ipc_num_bytes_read > 0 {
            println!("Received DFGM IPC Msg bytes");
            let deserialized_msg_result = deserialize_msg(&dfgm_ipc_initial_buf.as_slice());
            match deserialized_msg_result {
                Ok(deserialized_msg) => {
                    todo!()
                }
                Err(e) => {
                    println!("Error deserializing GS IPC msg: {:?}", e);
                    //Handle deserialization of IPC msg failure
                }
            };
        }
    }
    Ok(())
}
