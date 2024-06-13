/*  Written by: Rowan Rasmusson

    References: https://www.geeksforgeeks.org/process-schedulers-in-operating-system/
        - Justification for having multiple message states

    Saved_commands: name of the file that is created is the time of execution of the command
*/


use std::{time::Duration, io::{self}, sync::{Arc, Mutex}};
use std::sync::mpsc;
use std::thread;
pub mod schedule_message;
use crate::schedule_message::*;
pub mod scheduler;
use crate::scheduler::*;
pub mod log;
use crate::log::*;
use interfaces;
use message_structure::*;
use std::io::Cursor;

const CHECK_DELAY: u8 = 100;

fn main() {
    init_logger();

    let input: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    // let _command_queue: Arc<Mutex<String>> = input.clone();

    // Check commands while reading input
    thread::spawn(move || loop {
        let curr_time = get_current_time_millis();
        process_saved_commands("scheduler/saved_commands", curr_time);
        thread::sleep(Duration::from_millis(CHECK_DELAY as u64));
    });

    loop {
    // read in byte stream
    let ip = "127.0.0.1".to_string();
    let port = 8081; // change to msg dispatcher port
    let tcp_interface = interfaces::TcpInterface::new_server(ip, port).unwrap();

    let (sched_tx, sched_rx) = mpsc::channel();
    // make a message struct out of it. from_bytes() from message_structure
    interfaces::async_read(tcp_interface.clone(), sched_tx, 128);
    let buf = Vec::new();
    let mut cursor = Cursor::new(buf);
    let curr_msg_body = Vec::new();
    let deserialized_msg: Msg = Msg::new(0,0,0,0,curr_msg_body.clone());

    if let Ok(msg) = sched_rx.recv() {
        let deserialized_msg: Msg = serde_json::from_reader(&mut cursor).unwrap();

    } else {
        log_error("Could not receive message".to_string(), 5);
    }

    // read time as UNIX EPOCH u64
    let command_time: u64 = get_time(deserialized_msg.msg_body);
    let curr_time_millis: u64 = get_current_time_millis();

    let input_tuple: (u64, u8) = (command_time.clone(),deserialized_msg.header.msg_id);

    println!("Command Time: {:?} ms, ID: {}Current time is {:?} ms", input_tuple.0, input_tuple.1, curr_time_millis);

    // dummy message
    // won't need this when message is built

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

    }

    // Update the shared input
    let mut shared_input: std::sync::MutexGuard<'_, String> = input.lock().unwrap();
    *shared_input = deserialized_msg.header.msg_id.to_string();



}}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self};

    #[test]
    fn test_write_input_tuple_creates_file() {
        let test_dir = "scheduler/saved_commands".to_string();
        let input_tuple: (u64, u8) = (1717110630000, 30);

        let result = write_input_tuple_to_rolling_file(&input_tuple);
        assert!(result.is_ok());

        let files: Vec<_> = fs::read_dir(&test_dir).unwrap().collect();
        assert!(files.len() != 0);

    }

    #[test]
    fn test_oldest_file_deletion() {
        let test_dir = "scheduler/saved_commands";
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