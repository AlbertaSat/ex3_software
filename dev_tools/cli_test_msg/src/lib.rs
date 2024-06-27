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

            let date_time_utc = Utc.from_utc_datetime(&naive_date_time);
            let epoch_millis = date_time_utc.timestamp_millis();
            Ok(epoch_millis as u64)
        }
        Err(e) => Err(format!("Failed to parse timestamp: {}", e)),
    }
}