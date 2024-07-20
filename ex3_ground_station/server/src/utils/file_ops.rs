use crate::types::command::{Command, NewCommand};
use rocket::serde::json::serde_json;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::sync::Mutex;

static FILE_PATH: &str = "data/commands.json";

lazy_static::lazy_static! {
    static ref FILE_LOCK: Mutex<()> = Mutex::new(());
}

pub fn read_commands() -> io::Result<Vec<Command>> {
    let _lock = FILE_LOCK.lock().unwrap();
    if !Path::new(FILE_PATH).exists() {
        return Ok(Vec::new());
    }

    let file = File::open(FILE_PATH)?;
    let reader = BufReader::new(file);
    let commands: Vec<Command> = serde_json::from_reader(reader)?;

    Ok(commands)
}

pub fn write_command(new_command: NewCommand) -> io::Result<()> {
    let mut commands = read_commands()?;
    let next_id = commands.iter().map(|c| c.id).max().unwrap_or(0) + 1;

    let command = Command {
        id: next_id,
        payload: new_command.payload,
        cmd: new_command.cmd,
        data: new_command.data,
        timestamp: new_command.timestamp,
    };

    commands.push(command);
    
    let _lock: std::sync::MutexGuard<()> = FILE_LOCK.lock().unwrap();

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(FILE_PATH)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &commands)?;

    Ok(())
}