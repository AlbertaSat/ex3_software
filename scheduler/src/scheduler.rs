use std::fs;
use std::fs::File;
use std::path::Path;
use std::io;
use std::io::Write;
use chrono::{NaiveDateTime, TimeZone, Utc};

pub fn timestamp_to_epoch(timestamp: String) -> Result<u64, String> {
    // Split the input timestamp into date and time parts
    let parts: Vec<&str> = timestamp.split(' ').collect();
    if parts.len() != 2 {
        return Err("Invalid timestamp format".to_string());
    }

    let date_str = parts[0];
    let time_str = parts[1];

    // Parse the input date and time separately using NaiveDateTime from the chrono crate
    match NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time_str), "%Y-%m-%d %H:%M:%S") {
        Ok(naive_date_time) => {
            // Convert the NaiveDateTime to a DateTime<Utc> object
            let date_time_utc = Utc.from_utc_datetime(&naive_date_time);
            // Get the epoch timestamp in milliseconds
            let epoch_millis = date_time_utc.timestamp_millis();
            // Convert the epoch timestamp to u64
            Ok(epoch_millis as u64)
        }
        Err(e) => Err(format!("Failed to parse timestamp: {}", e)),
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