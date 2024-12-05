use std::str;

use common::component_ids::ComponentIds;
use common::message_structure::Msg;

pub fn parse_cmd(input: &[&str]) -> Option<Vec<u8>> {
    if input[0] == "help" || input[0] == "?" {
        println!("Usage: {} <any linux cmd>", ComponentIds::SHELL);
        None
    }
    else {
        Some(input.join(" ").as_bytes().to_vec())
    }
}

pub fn handle_response(msg: &Msg) {
    match str::from_utf8(&msg.msg_body) {
        Ok(reply) => println!("{}", reply),
        Err(e) => eprintln!("Shell reply error: {}", e),
    }
}

