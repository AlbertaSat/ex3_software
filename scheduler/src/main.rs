/*  Written by: Rowan Rasmusson

    References: https://www.geeksforgeeks.org/process-schedulers-in-operating-system/
        - Justification for having multiple message states

    Saved_commands: name of the file that is created is the time of execution of the command
*/


use std::{time::Duration, sync::{Arc, Mutex}};
use std::sync::mpsc;
use std::thread;
pub mod schedule_message;
use crate::schedule_message::*;
pub mod scheduler;
use crate::scheduler::*;
pub mod log;
use crate::log::*;
use interfaces::{self, TCP_BUFFER_SIZE};
use message_structure::*;
use common::{self, ports::SCHEDULER_DISPATCHER_PORT};
use std::io::Cursor;
const CHECK_DELAY: u8 = 100;

fn main() {
    init_logger();
    check_saved_commands();
    run_scheduler();
}

fn check_saved_commands() {
    thread::spawn(move || loop {
        let curr_time = get_current_time_millis();
        process_saved_commands("saved_commands", curr_time);
        thread::sleep(Duration::from_millis(CHECK_DELAY as u64));
    });
}

fn run_scheduler() {

    let input: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let ip = "127.0.0.1".to_string();
    let port = SCHEDULER_DISPATCHER_PORT;
    let tcp_interface = interfaces::TcpInterface::new_server(ip, port).unwrap();

    let (sched_reader_tx, sched_reader_rx) = mpsc::channel();
    // let (sched_writer_tx, sched_writer_rx) = mpsc::channel();

    interfaces::async_read(tcp_interface.clone(), sched_reader_tx, TCP_BUFFER_SIZE);

    loop {
        if let Ok(buffer) = sched_reader_rx.recv() {
            // Trimming trailing 0's so JSON doesn't give a "trailing characters" error
            let trimmed_buffer: Vec<_> = buffer.into_iter().take_while(|&x| x != 0).collect();
            let mut cursor = Cursor::new(trimmed_buffer);
            let deserialized_msg: Msg = serde_json::from_reader(&mut cursor).unwrap();

            // gets execution time as unix epoch
            let command_time: u64 = get_time(deserialized_msg.msg_body);
            let curr_time_millis: u64 = get_current_time_millis();
            let input_tuple: (u64, u8) = (command_time.clone(),deserialized_msg.header.msg_id);

            println!("Command Time: {:?} ms, ID: {} Current time is {:?} ms", input_tuple.0, input_tuple.1, curr_time_millis);

            if command_time <= curr_time_millis {
            // Message state dictates what the scheduler does with the message
            // msg.state = MessageState::Running;
            // handle_state(&msg);
                log_error("Received command from past".to_string(), deserialized_msg.header.msg_id);
            } else {
                // save to non-volatile memory
                if let Err(err) = write_input_tuple_to_rolling_file(&input_tuple) {
                    eprintln!("Failed to write input tuple to file: {}", err);
                } else {
                    println!("Input tuple written to file successfully.");
                }
            log_info("Command stored and scheduled for later".to_string(), deserialized_msg.header.msg_id);

            // Update the shared input
            let mut shared_input: std::sync::MutexGuard<'_, String> = input.lock().unwrap();
            *shared_input = deserialized_msg.header.msg_id.to_string();
        }
    } else {
        log_error("Could not receive message".to_string(), 5);
            }
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self};

    #[test]
    fn test_write_input_tuple_creates_file() {
        let test_dir = "saved_commands".to_string();
        let input_tuple: (u64, u8) = (1717110630000, 30);

        let result = write_input_tuple_to_rolling_file(&input_tuple);
        assert!(result.is_ok());

        let files: Vec<_> = fs::read_dir(&test_dir).unwrap().collect();
        assert!(files.len() != 0);

    }

    #[test]
    fn test_oldest_file_deletion() {
        let test_dir = "saved_commands";
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