use log::info;

#[derive(Clone)]
pub enum MessageState {
    New,
    Suspended,
    Waiting,
    Running,
    Done,
}
#[derive(Clone)]
pub struct Message {
    pub time: Result<u64, String>,
    pub state: MessageState,
    pub id: u32,
    pub command: String,
}

pub fn handle_state(msg: &Message) {
    match msg.state {
        MessageState::New => {
            info!("New task received at {:?}. #{}",msg.time, msg.id);
        }
        MessageState::Running => {
            info!("Task #{} is running.", msg.id);
            println!("Sent to CmdDispatcher");
            // Send to CmdDispatcher
        }
        MessageState::Done => {
            info!("Task #{} is done running.", msg.id)
            // Acknowledgement from CmdDispatcher
        }
        MessageState::Waiting => {
            info!("Task #{} is waiting to run.", msg.id);
            // Put into CmdDispatcher buffer
        }
        MessageState::Suspended => {
            info!("Task #{} is suspended.", msg.id);
            // Put into non-volatile memory
        }
    }
}

// This function exists so the message is able to be scheduled
pub fn get_time(msg_body: Vec<u8>) -> u64 {
    assert!(msg_body.len() >= 6, "msg_body must be at least 6 bytes long");
    let time_bytes = &msg_body[0..6];

    // Convert bytes to a u64 (assuming little-endian order)
    let mut time: u64 = 0;
    for (i, &byte) in time_bytes.iter().enumerate() {
        time |= (byte as u64) << (i * 8);
    }

    time
}