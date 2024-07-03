/*  Written by: Rowan Rasmusson

    References: https://www.geeksforgeeks.org/process-schedulers-in-operating-system/
        - Justification for having multiple message states

    Saved_messages: name of the file that is created is the time of execution of the command
*/


use std::{time::Duration, sync::{Arc, Mutex}, collections::HashSet};
use std::sync::mpsc;
use std::thread;
pub mod schedule_message;
use crate::schedule_message::*;
pub mod scheduler;
use crate::scheduler::*;
pub mod log;
use crate::log::*;
use tcp_interface::{self, TCP_BUFFER_SIZE};
use message_structure::*;
use common::{self, ports::SCHEDULER_DISPATCHER_PORT};
const CHECK_DELAY: u8 = 100;

fn main() {
    init_logger();
    check_saved_messages();
    run_scheduler();
}

fn check_saved_messages() {
    let already_read = Arc::new(Mutex::new(HashSet::new()));

    thread::spawn(move || loop {
        let already_read_clone = Arc::clone(&already_read);
        let curr_time = get_current_time_millis();
        process_saved_messages("scheduler/saved_messages", curr_time, &already_read_clone);
        thread::sleep(Duration::from_millis(CHECK_DELAY as u64));
    });
}

fn run_scheduler() {
    let input: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    let ip = "127.0.0.1".to_string();
    let port = SCHEDULER_DISPATCHER_PORT;
    loop {


        let tcp_interface = tcp_interface::TcpInterface::new_server(ip.clone(), port).unwrap();

        let (sched_reader_tx, sched_reader_rx) = mpsc::channel();
        // let (sched_writer_tx, sched_writer_rx) = mpsc::channel();

        tcp_interface::async_read(tcp_interface.clone(), sched_reader_tx, TCP_BUFFER_SIZE);

        match sched_reader_rx.recv() {
            Ok(buffer) => {
                let deserialized_msg: Msg = deserialize_msg(buffer).unwrap();
                process_message(deserialized_msg, &input);
            }
            Err(e) => {
                // ID 5 is arbitrary ID for error message
                log_error(format!("Failed to deserialize message {}", e), 5);
            }
        }
    }
}

fn process_message(deserialized_msg: Msg, input: &Arc<Mutex<String>>) {
    // unwrap message to get inner message for the subsystem
    // the message body is the serialized message
    let subsystem_msg: Msg = deserialize_msg(deserialized_msg.msg_body).unwrap();

    let command_time: u64 = get_time(subsystem_msg.msg_body);
    let curr_time_millis: u64 = get_current_time_millis();
    let input_tuple: (u64, u8) = (command_time, subsystem_msg.header.msg_id);

    println!("Command Time: {:?} ms, ID: {} Current time is {:?} ms", input_tuple.0, input_tuple.1, curr_time_millis);

    if command_time <= curr_time_millis {
        log_error("Received command from past".to_string(), deserialized_msg.header.msg_id);
    } else {
        if let Err(err) = write_input_tuple_to_rolling_file(&input_tuple) {
            eprintln!("Failed to write input tuple to file: {}", err);
        } else {
            println!("Input tuple written to file successfully.");
        }
        log_info("Command stored and scheduled for later".to_string(), deserialized_msg.header.msg_id);

        let mut shared_input = input.lock().unwrap();
        *shared_input = deserialized_msg.header.msg_id.to_string();
    }
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