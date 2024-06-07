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