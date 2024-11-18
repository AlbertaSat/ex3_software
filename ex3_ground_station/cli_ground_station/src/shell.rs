use common::component_ids::ComponentIds;

pub fn parse_cmd(input: &[&str]) -> Option<Vec<u8>> {
    if input[0] == "help" || input[0] == "?" {
        println!("Usage: {} <any linux cmd>", ComponentIds::SHELL);
        None
    }
    else {
        Some(input.join(" ").as_bytes().to_vec())
    }
}
                
