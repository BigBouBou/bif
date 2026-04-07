//Command enum + parsing from args
use crate::cli;

pub enum Command {
    HELP,
    // Shows the help message
    INIT,
    // Intialises a new .bif file.
    TRACK,
    // Tracks an existing .bif file.
    NEW,
    //Create a new entry.
    DELETE,
    // Deletes the last entry, or the selected entry
    READ,
    // Reads the current .bif file in its entirety
}

impl Command {
    pub fn parse(input: &Vec<String>) -> Option<Command> {
        if input.is_empty() {
            return Some(Command::HELP);
        }
        match input[0].as_str() {
            "help" => Some(Command::HELP),
            "init" => Some(Command::INIT),
            "track" => Some(Command::TRACK),
            "new" => Some(Command::NEW),
            "delete" => Some(Command::DELETE),
            "read" => Some(Command::READ),
            _ => None,
        }
    }

    pub fn execute(&self) {
        match self {
            Command::HELP => {
                cli::help::render();
            }
            Command::INIT => {
                println!("not implemented yet")
            }
            Command::TRACK => {
                println!("not implemented yet")
            }
            Command::NEW => {
                println!("not implemented yet")
            }
            Command::DELETE => {
                println!("not implemented yet")
            }
            Command::READ => {
                println!("not implemented yet")
            }
        }
    }
}
