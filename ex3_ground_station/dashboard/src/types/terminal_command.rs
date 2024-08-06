pub struct TerminalCommand {
  name: String,
  arguments: Vec<String>,
}

impl TerminalCommand {
  pub fn new(name: &str, arguments: Vec<String>) -> Self {
    TerminalCommand {
      name: name.to_string(),
      arguments,
    }
  }

  pub fn execute(&self) -> String {
    match self.name.as_str() {
      "help" => self.help(),
      "echo" => self.echo(),
      "clear" => self.clear(),
      _ => format!("Command '{}' not found.", self.name),
    }
  }

  fn help(&self) -> String {
    "Available commands: help, echo, clear".to_string()
  }

  fn echo(&self) -> String {
    self.arguments.join(" ")
  }

  fn clear(&self) -> String {
    "Screen cleared.".to_string()
  }
}

