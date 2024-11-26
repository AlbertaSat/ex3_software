
pub fn parse_cmd(input: &[&str]) -> Option<Vec<u8>> {
    match input.len() {
        0 => {
            println!("Missing EPS command");
            None
        },
        _ => {
            Some(input.join(" ").as_bytes().to_vec())
        }
    }
}
