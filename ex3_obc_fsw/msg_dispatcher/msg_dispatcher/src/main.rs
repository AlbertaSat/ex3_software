// Written by Tomasz Ayobahan
// Summer 2024

use ipc::*;
use std::io::Result;
// use std::process::exit;

fn main() -> Result<()> {

    // All connected handlers and other clients will have a socket for the server defined here
    let mut iris_handler: IpcServer = IpcServer::new("iris_disp".to_string())?;
    let mut dfgm_handler: IpcServer = IpcServer::new("dfgm_disp".to_string())?;
    let mut coms_handler: IpcServer = IpcServer::new("coms_disp".to_string())?;
    let mut test_handler: IpcServer = IpcServer::new("test_disp".to_string())?;

    // let mut components: Vec<&mut IpcServer> = vec![
    //     &mut iris_handler, &mut dfgm_handler, &mut coms_handler, &mut test_handler
    //     ];

    loop {
        
        let mut components: Vec<&mut IpcServer> = vec![
            &mut iris_handler, &mut dfgm_handler, &mut coms_handler, &mut test_handler
        ];

        poll_ipc_server_sockets(&mut components);

        for component in components {

            if read_data_socket(component) == 0 {
                continue;
            }

            if component.buffer.starts_with(b"DOWN") {
                println!("Received DOWN - server shutting down");
                // exit(0);
                return Ok(());
            }

            let dest_id = get_msg_dest_id(&component.buffer);
            let dest_comp_fd = get_fd_from_id(components, dest_id);
            if dest_comp_fd > -1 {
                ipc_write(Some(dest_comp_fd), &component.buffer);
            }
            component.clear_buffer() // clear read buffer after handling data

        }
    }

    // fn goto_clean_end() {
    //     for component in components {
    //         // Clean up resources associated with components
    //     }
    //     exit(0);
    // }
}

// fn handle_error(error_msg: &str) -> ! {
//     eprintln!("Error: {}", error_msg);
//     exit(1);
// }

fn get_msg_dest_id(data_buf: &[u8]) -> i32 {
    let dest_id = data_buf[2] as i32;
    println!("Msg Dest ID: {}", dest_id);
    dest_id
}

fn get_fd_from_id(components: Vec<&mut IpcServer>, id: i32) -> i32 {
    for component in components {
        if component.data_fd == Some(id) {
            println!("Component match : {}, with id: {}", component.socket_path, component.data_fd.unwrap());

            // if !component.connected {
            //     println!("Component not connected. Not writing");
            //     return -2;
            // }
            println!("Destination component fd: {}", id);
            return component.data_fd.unwrap();
        }
    }
    println!("No matching component found with id: {}", id);
    -1
}

fn read_data_socket(component: &mut IpcServer) -> i32 {
    let ret = component.data_fd.unwrap();
    if ret == 0 {
        // component.client_disconnected(); need to close connection?
        println!("Connection to socket: {} closed. (zero byte read indicates this)", component.socket_path);
        component.clear_buffer();
    } else {
        println!("---------------------------------------");
        println!("Read {} bytes:", ret);
        println!("Data in HEX is:");
        for byte in &component.buffer {
            print!(" {:02x} |", byte);
        }
        println!("\n---------------------------------------");
    }
    ret as i32
}

