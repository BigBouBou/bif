use crate::cli::cli_error::CliError;
use crate::{cli, domain, storage};
pub enum Command {
    HELP,
    // Shows the help message
    INIT { name_of_new_log: Option<String> },
    // Intialises a new .bif file.
    TRACK,
    // Tracks an existing .bif file.
    NEW { body: String },
    //Create a new entry.
    DELETE,
    // Deletes the last entry, or the selected entry
    READ,
    // Reads the current .bif file in its entirety
}

impl Command {
    /// Parses user input into a command.
    pub fn parse(input: &Vec<String>) -> Option<Command> {
        if input.is_empty() {
            return Some(Command::HELP);
        }

        match input[0].as_str() {
            "help" => Some(Command::HELP),
            "init" => {
                // bif init <optionalName>
                // - No name => default Record.bif
                // - One arg => use it
                // - More than one => invalid
                match input.len() {
                    1 => Some(Command::INIT {
                        name_of_new_log: None,
                    }),
                    2 => Some(Command::INIT {
                        name_of_new_log: Some(input[1].clone()),
                    }),
                    _ => None,
                }
            }
            "track" => Some(Command::TRACK),
            "new" => Some(Command::NEW {
                body: String::new(),
            }),
            "delete" => Some(Command::DELETE),
            "read" => Some(Command::READ),
            _ => None,
        }
    }

    /// Executes a command.
    pub fn execute(&self) -> Result<(), CliError> {
        match self {
            Command::HELP => {
                cli::help::render();
                Ok(())
            }

            Command::INIT { name_of_new_log } => {
                // Normalize/validate at the domain boundary.
                let file_name =
                    domain::log_filename::normalize_log_filename(name_of_new_log.as_deref())?;

                // Storage operation: create the file in the current working directory.
                //
                // For now `fs_store` returns `std::io::Result`, so we map it into a
                // `StorageError::Io` via `From<std::io::Error> for StorageError`,
                // and then into `CliError::Storage` via `From<StorageError> for CliError`.
                //
                // Later, you can change `fs_store` to return `Result<_, StorageError>`
                // and delete this explicit mapping.
                let created_path = storage::fs_store::create_empty_record_file_in_cwd(&file_name)
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                println!("Initialized empty record: {}", created_path.display());
                Ok(())
            }

            Command::TRACK => {
                println!("not implemented yet");
                Ok(())
            }

            Command::NEW { body: _ } => {
                println!("not implemented yet");
                Ok(())
            }

            Command::DELETE => {
                println!("not implemented yet");
                Ok(())
            }

            Command::READ => {
                println!("not implemented yet");
                Ok(())
            }
        }
    }
}
