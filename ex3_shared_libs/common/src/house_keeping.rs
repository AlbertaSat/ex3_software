// This file includes some helper functions that create and handle json structures for housekeeping
// purposes on the satellite.

use serde_json::{Value, json};
use std::fs;
use std::io::{BufReader, BufWriter};
use crate::component_ids::ComponentIds;
use chrono::prelude::*;

pub struct HKData {
    json: Value
}

impl HKData {
    pub fn new(id: ComponentIds) -> Self {
        let utc_time = Utc::now().to_string();
        let json = json!({
            "TIME": utc_time,
            "SUBSYSTEM": id as u8,
        });
        HKData{json}
    }

    pub fn key_value_pair(&mut self, key: &str, value: Value) {
        self.json[key] = value;
    }

    /// This function writes a JSON value to a file.
    /// If the filepath given as argument does not exist, it will be created, if the file already
    /// exists this function will completely overwrite the previous data with the new json object data.
    pub fn write_to_file(&mut self, filepath: &str) ->Result<(), std::io::Error> {
        let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(filepath)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self.json)?;
        Ok(())
    }

    pub fn from_json(filepath: &str) -> Result<Self, std::io::Error> {
        let file = fs::File::open(filepath)?;
        let reader = BufReader::new(file);
        let hk = serde_json::from_reader(reader)?;
        // could be any component id because its going to be overwritten anyways
        let mut hk_struct = HKData::new(ComponentIds::SHELL);
        hk_struct.json = hk;
        Ok(hk_struct)
    }
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

        let mut my_hk = HKData::new(ComponentIds::DFGM);
        let _ = my_hk.write_to_file(path.to_str().unwrap());
    }

    #[test]
    fn test_read_hk() {
        let tmp_dir = TempDir::new("hk_dir").unwrap();
        let path = tmp_dir.path().join("dfgm.JSON");

        let mut my_hk = HKData::new(ComponentIds::DFGM);
        let _ = my_hk.write_to_file(path.to_str().unwrap());
        let _ = read_hk(path.to_str().unwrap()).unwrap();
    }

    #[test]
    fn test_create_hk() {
        let mut my_hk = HKData::new(ComponentIds::DFGM);
        my_hk.key_value_pair("test data", [3,4,5,6].into());
        assert!(3 == my_hk.json["test data"][0]);
    }
}
