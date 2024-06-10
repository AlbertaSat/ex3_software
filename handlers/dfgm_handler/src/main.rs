/*
Written By Devin Headrick
Summer 2024

Handler will    
    - handle all communication with their respective subsystem via interface 
    - communicate with other FSW via IPC 
    - take opcodes from messages passed to them, and execute their related functionality


DFGM is a simple subsystem that only outputs a ~1250 byte packet at 1Hz, with no interface or control from the FSW. 
The handler either chooses to collect the data or not. 

*/

/// Opcodes for messages relating to DFGM functionality
pub enum OpCode {
   ToggleDataCollection, // toggles a flag which either enables or disables data collection from the DFGM 
}

struct DFGMHandler {
    toggle_data_collection: bool, 
}

/// on startup get value of toggle_data_collection from the State manager 

fn main() {
    println!("Beginning DFGM Handler...");
    
}


