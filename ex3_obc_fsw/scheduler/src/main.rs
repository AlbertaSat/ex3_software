// /*  Written by: Rowan Rasmusson

//     References: https://www.geeksforgeeks.org/process-schedulers-in-operating-system/
//         - Justification for having multiple message states

//     Saved_messages: name of the file that is created is the time of execution of the command
// */

// TMP for cargo build

use std::{collections::HashSet, sync::{Arc, Mutex}, time::Duration};
use std::thread;
pub mod schedule_message;
use crate::schedule_message::*;
pub mod scheduler;
use crate::scheduler::*;
use message_structure::*;

use ipc::{IpcClient, IpcServer, IPC_BUFFER_SIZE, poll_ipc_server_sockets};
use log::{debug, trace, warn};
use logging::init_logger;

const CHECK_DELAY: u8 = 100;

struct Scheduler {
    cmd_dispatcher_interface: Option<IpcServer>,
    coms_handler_resp_interface: Option<IpcClient>
}

impl Scheduler {
    fn new(
        cmd_dispatcher_interface: Result<IpcServer, std::io::Error>,
        coms_handler_resp_interface: Result<IpcClient, std::io::Error>,
    ) -> Scheduler {
        if cmd_dispatcher_interface.is_err() {
            warn!(
                "Error creating dispatcher interface: {:?}",
                cmd_dispatcher_interface.as_ref().err().unwrap()
            );
        }
        if coms_handler_resp_interface.is_err() {
            warn!(
                "Error creating coms interface: {:?}",
                coms_handler_resp_interface.as_ref().err().unwrap()
            );
        }
        Scheduler {
            cmd_dispatcher_interface: cmd_dispatcher_interface.ok(),
            coms_handler_resp_interface: coms_handler_resp_interface.ok(),
        }
    }

    fn run(&mut self) -> std::io::Result<()> {
        // start thread for checking execution time of scheduled commands
        check_saved_messages();
        // Poll for messages
        loop {
            // First, take the Option<IpcClient> out of `self.dispatcher_interface`
            // This consumes the Option, so you can work with the owned IpcClient
            let cmd_dispatcher_interface = self.cmd_dispatcher_interface.take().expect("Cmd_Disp interface has value of None");

            // Create a mutable Option<IpcClient> so its lifetime persists
            let mut cmd_dispatcher_interface_option = Some(cmd_dispatcher_interface);

            // Now you can borrow this mutable option and place it in the vector
            let mut server: Vec<&mut Option<IpcServer>> = vec![
                &mut cmd_dispatcher_interface_option,
            ];

            poll_ipc_server_sockets(&mut server);

            // restore the value back into `self.dispatcher_interface` after polling. May have been mutated
            self.cmd_dispatcher_interface = cmd_dispatcher_interface_option;

            // Handling the bulk message dispatcher interface
            let msg_dispatcher_interface = self.cmd_dispatcher_interface.as_ref().unwrap();
            if msg_dispatcher_interface.buffer != [0u8; IPC_BUFFER_SIZE] {
                let recv_msg: Msg = deserialize_msg(&msg_dispatcher_interface.buffer).unwrap();
                debug!("Received and deserialized msg");
                self.handle_msg(recv_msg)?;
            }

        }
    }

    fn handle_msg(&mut self, msg: Msg) -> Result<(), std::io::Error> {
        self.cmd_dispatcher_interface.as_mut().unwrap().clear_buffer();

        process_message(msg, &Arc::new(Mutex::new(String::new())));

        Ok(())
    }


}

// fn main() {
//     init_logger();
//     check_saved_messages();
//     run_scheduler();
// }

fn check_saved_messages() {
    let already_read = Arc::new(Mutex::new(HashSet::new()));

    thread::spawn(move || loop {
        let already_read_clone = Arc::clone(&already_read);
        let curr_time = get_current_time_millis();
        process_saved_messages("scheduler/saved_messages", curr_time, &already_read_clone);
        thread::sleep(Duration::from_millis(CHECK_DELAY as u64));
    });
}

// fn run_scheduler() {
//     let input: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

//     let ip = "127.0.0.1".to_string();
//     let port = SCHEDULER_DISPATCHER_PORT;
//     loop {

//         let tcp_interface = tcp_interface::TcpInterface::new_server(ip.clone(), port).unwrap();

//         let (sched_reader_tx, sched_reader_rx) = mpsc::channel();
//         // let (sched_writer_tx, sched_writer_rx) = mpsc::channel();

//         tcp_interface::async_read(tcp_interface.clone(), sched_reader_tx, TCP_BUFFER_SIZE);

//         let tcp_interface = tcp_interface::TcpInterface::new_server(ip.clone(), port).unwrap();

//         let (sched_reader_tx, sched_reader_rx) = mpsc::channel();
//         // let (sched_writer_tx, sched_writer_rx) = mpsc::channel();

//         tcp_interface::async_read(tcp_interface.clone(), sched_reader_tx, TCP_BUFFER_SIZE);
//         match sched_reader_rx.recv() {
//             Ok(buffer) => {
//                 let deserialized_msg: Msg = deserialize_msg(buffer).unwrap();
//                 process_message(deserialized_msg, &input);
//             }
//             Err(e) => {
//                 // ID 5 is arbitrary ID for error message
//                 log_error(format!("Failed to deserialize message {}", e), 5);
//             }
//         }
//     }
// }

fn process_message(deserialized_msg: Msg, input: &Arc<Mutex<String>>) {
    // unwrap message to get inner message for the subsystem
    // the message body is the serialized message

    let subsystem_msg = deserialize_msg(&deserialized_msg.msg_body).unwrap();

    let command_time: u64 = get_time(subsystem_msg.msg_body);
    let curr_time_millis: u64 = get_current_time_millis();
    let input_tuple: (u64, u8) = (command_time, subsystem_msg.header.msg_id);

    println!("Command Time: {:?} ms, ID: {} Current time is {:?} ms", input_tuple.0, input_tuple.1, curr_time_millis);

    if command_time <= curr_time_millis {
        trace!("Received command from past: ID {}", deserialized_msg.header.msg_id);
    } else {
        if let Err(err) = write_input_tuple_to_rolling_file(&input_tuple) {
            eprintln!("Failed to write input tuple to file: {}", err);
        } else {
            println!("Input tuple written to file successfully.");
        }
        trace!("Command ID: {} stored and scheduled for later", deserialized_msg.header.msg_id);

        let mut shared_input = input.lock().unwrap();
        *shared_input = deserialized_msg.header.msg_id.to_string();
    }
}

fn main() {
    let log_path = "ex3_obc_fsw/scheduler/logs";
    init_logger(log_path);

    trace!("Starting Scheduler...");

    // Create Unix domain socket interface for to talk to message dispatcher
    let cmd_dispatcher_interface = IpcServer::new("SCHEDULER".to_string());

    let gs_interface = IpcClient::new("gs_non_bulk".to_string());

    let mut scheduler = Scheduler::new(cmd_dispatcher_interface, gs_interface);

    let _ = scheduler.run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self};

    #[test]
    fn test_write_input_tuple_creates_file() {
        let test_dir = "scheduler/saved_messages".to_string();
        let input_tuple: (u64, u8) = (1717110630000, 30);

        let result = write_input_tuple_to_rolling_file(&input_tuple);
        assert!(result.is_ok());

        let files: Vec<_> = fs::read_dir(&test_dir).unwrap().collect();
        assert!(files.len() != 0);

    }

    #[test]
    fn test_oldest_file_deletion() {
        let test_dir = "scheduler/saved_messages";
        fs::create_dir_all(test_dir).unwrap();

        let input_tuple = (1717428208, 66);

        // Create files to exceed the max size
        for i in 0..2000 {
            let new_timestamp: u64 = input_tuple.0.clone() + i;
            write_input_tuple_to_rolling_file(&(new_timestamp, input_tuple.1.clone())).unwrap();
        }

        // Check initial number of files
        let initial_files: Vec<_> = fs::read_dir(test_dir).unwrap().collect();

        // Write an input tuple to trigger the removal of the oldest file
        let input_tuple = (2717428208, 77);
        write_input_tuple_to_rolling_file(&(input_tuple.0, input_tuple.1)).unwrap();

        // Check final number of files
        let final_files: Vec<_> = fs::read_dir(test_dir).unwrap().collect();
        assert_eq!(final_files.len(), initial_files.len());

        // Ensure the oldest file was removed
        let files: Vec<_> = fs::read_dir(test_dir)
            .unwrap()
            .map(|res| res.unwrap().file_name().into_string().unwrap())
            .collect();
        assert!(!files.contains(&String::from("1717428208.txt")));

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();
    }

}