use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Command {
    pub id: Option<i32>,
    pub payload: String,
    pub cmd: String,
    pub data: String,
    pub timestamp: String,
}
