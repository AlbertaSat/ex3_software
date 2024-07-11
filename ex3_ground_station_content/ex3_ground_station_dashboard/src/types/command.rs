#[derive(Default, Clone, PartialEq)]
pub struct Command {
    pub payload: String,
    pub cmd: String,
    pub data: String,
}