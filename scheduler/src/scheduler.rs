use std::fs;
use std::fs::File;
use std::path::Path;
use std::io;
use std::io::Write;
use chrono::{NaiveDateTime, TimeZone, Utc};
use std::time::SystemTime;
use std::io::BufRead;
use crate::{log_info, log_error};

pub fn process_saved_commands(dir: &str, curr_time_millis: u64) {
    let saved_commands_dir = Path::new(dir);
    if saved_commands_dir.exists() && saved_commands_dir.is_dir() {
        match fs::read_dir(saved_commands_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    process_entry(entry, curr_time_millis);
                }
            }
            Err(e) => eprintln!("Error reading directory: {:?}", e),
        }
    } else {
        log_error("Directory does not exist or is not a directory.".to_string(), 54);
    }
}

fn process_entry(entry: fs::DirEntry, curr_time_millis: u64) {
    if let Some(file_name) = entry.file_name().to_str() {
        if file_name.ends_with(".txt") {
            if let Ok(file_time) = file_name.trim_end_matches(".txt").parse::<u64>() {
                if file_time <= curr_time_millis {
                    send_command(&entry.path(), file_name);
                }
            }
        }
    }
}

fn send_command(file_path: &Path, file_name: &str) {
    match fs::File::open(file_path) {
        Ok(file) => {
            let lines: Vec<String> = io::BufReader::new(file)
                .lines()
                .filter_map(Result::ok)
                .collect();
            if lines.len() > 1 {
                println!("Sent Command {} at {}", lines[1], file_name);
            } else {
                println!("File {} does not have a command.", file_name);
            }
            log_info(format!("Processed file: {}", file_name), 0); // Message ID can be managed as needed
        }
        Err(e) => eprintln!("Failed to open file {}: {:?}", file_name, e),
    }
}

pub fn get_current_time_millis() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(e) => {
            eprint!("Error {:?}", e);
            0
        }
    }
}

fn delete_file(file_path: &Path) {
    match fs::remove_file(file_path) {
        Ok(_) => println!("Deleted file: {:?}", file_path),
        Err(e) => eprintln!("Failed to delete file {:?}: {:?}", file_path, e),
    }
}

pub fn write_input_tuple_to_rolling_file(input_tuple: &(Result<u64, String>, String)) -> Result<(), io::Error> {
    // Create the directory if it doesn't exist
    let dir_path = "scheduler/saved_commands";
    fs::create_dir_all(dir_path)?;

    // Get the total size of files in the directory
    let total_size: u64 = fs::read_dir(dir_path)?
        .filter_map(|res| res.ok())
        .map(|entry| entry.metadata().ok().map(|m| m.len()).unwrap_or(0))
        .sum();

    // Specify the maximum size of saved_commands directory in bytes
    let max_size_bytes: u64 = 2048; // 2 KB

    // If the total size exceeds the maximum size, remove the oldest file
    if total_size >= max_size_bytes {
        remove_oldest_file(&dir_path)?;
    }

    // Create a new file
    let file_name = format!("{}.txt", input_tuple.0.as_ref().unwrap());
    let file_path = Path::new(dir_path).join(&file_name);
    let mut file = File::create(&file_path)?;

    // Write input_tuple to the file
    writeln!(file, "{:?}\n{}", input_tuple.0.as_ref().unwrap(), input_tuple.1)?;

    Ok(())
}

fn remove_oldest_file(dir_path: &str) -> Result<(), io::Error> {
    let oldest_file = fs::read_dir(dir_path)?
        .filter_map(|res| res.ok())
        .min_by_key(|entry: &fs::DirEntry| entry.metadata().unwrap().modified().unwrap());

    if let Some(oldest_file) = oldest_file {
        fs::remove_file(oldest_file.path())?;
    }

    Ok(())
}