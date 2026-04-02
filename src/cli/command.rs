//Command enum + parsing from args
use crate::cli;

pub enum Command {
    Help,
    // Shows the help message
    Init,
    // Intialises the current note chain (new note chain)
    // The app now points to a new note named "1"
    New,
    // Creates a new note in the current note chain
    // The app now points to the new note
    Append { text: String },
    // Appends content to the current note
    // The app now points to the same note
    Delete,
    // Deletes the current note
    // The app now points to the previous note
    Read,
    // Reads the current note, and shows its content in the CLI
    // The app now points to the same note
}

impl Command {
    pub fn parse(input: &Vec<String>) -> Option<Command> {
        if input.is_empty() {
            return Some(Command::Help);
        }
        match input[0].as_str() {
            "help" => Some(Command::Help),
            "init" => Some(Command::Init),
            "new" => Some(Command::New),
            "append" => Some(Command::Append {
                text: input[1].clone(),
            }),
            "delete" => Some(Command::Delete),
            "read" => Some(Command::Read),
            _ => None,
        }
    }

    pub fn execute(&self) {
        match self {
            Command::Help => {
                cli::help::render();
            }
            Command::Init => {}
            Command::New => {}
            Command::Append { text } => {}
            Command::Delete => {}
            Command::Read => {}
            _ => {}
        }
    }
}
