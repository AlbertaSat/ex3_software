use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Command {
    pub id: i32,
    pub payload: String,
    pub cmd: String,
    pub data: String,
    pub timestamp: String,
}

#[derive(Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct NewCommand {
    pub payload: String,
    pub cmd: String,
    pub data: String,
    pub timestamp: String,
}
