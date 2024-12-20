// This file includes some helper functions that create and handle json structures for housekeeping
// purposes on the satellite.

use serde_json::{Value, json};
use std::fs;
use std::io::{BufReader, BufWriter};
use crate::component_ids::ComponentIds;
use chrono::prelude::*;

/// Simple helper function that creates a json value initialized with
/// the component id of the system and the current UTC time.
pub fn create_hk(id: ComponentIds) -> Value {
    let utc_time = Utc::now().to_string();
    json!({
        "TIME": utc_time,
        "SUBSYSTEM": id as u8,
    })
}

/// This function writes a JSON value to a file.
/// If the filepath given as argument does not exist, it will be created, if the file already
/// exists this function will completely overwrite the previous data with the new json object data.
pub fn write_hk(filepath: &str, json_obj: &Value) -> Result<(), std::io::Error> {
    let file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(filepath)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, json_obj)?;
    Ok(())
}

/// This function reads a file and converts it into a JSON value, returns the JSON value.
pub fn read_hk(filepath: &str) -> Result<Value, std::io::Error> {
    let file = fs::File::open(filepath)?;
    let reader = BufReader::new(file);
    let hk = serde_json::from_reader(reader)?;
    Ok(hk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_write_hk() {
        let tmp_dir = TempDir::new("hk_dir").unwrap();
        let path = tmp_dir.path().join("dfgm.JSON");

        let my_hk = create_hk(ComponentIds::DFGM);
        write_hk(path.to_str().unwrap(), &my_hk).unwrap();
    }

    #[test]
    fn test_read_hk() {
        let tmp_dir = TempDir::new("hk_dir").unwrap();
        let path = tmp_dir.path().join("dfgm.JSON");

        let my_hk = create_hk(ComponentIds::DFGM);
        write_hk(path.to_str().unwrap(), &my_hk).unwrap();
        let _ = read_hk(path.to_str().unwrap()).unwrap();
    }

    #[test]
    fn test_create_hk() {
        let mut my_hk = create_hk(ComponentIds::DFGM);
        my_hk["test data"] = [3,4,5,6].into();
        assert!(3 == my_hk["test data"][0]);
    }
}
