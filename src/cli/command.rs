use crate::cli::cli_error::CliError;
use crate::{cli, domain, storage};

fn require_tracked_log() -> Result<String, CliError> {
    match storage::tracked::get_tracked_file_path() {
        Ok(path) => Ok(path),
        Err(_err) => Err(CliError::InvalidArgs {
            message: "no tracked log. Run `bif init` or `bif track <name>`.".to_string(),
        }),
    }
}
pub enum Command {
    HELP,
    // Shows the help message
    INIT { name_of_new_log: Option<String> },
    // Intialises a new .bif file.
    TRACK { name_of_log: String },
    // Tracks an existing .bif file in the current working directory.
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
            "track" => match input.len() {
                2 => Some(Command::TRACK {
                    name_of_log: input[1].clone(),
                }),
                _ => None,
            },
            "new" => {
                // `bif new <body...>`
                if input.len() < 2 {
                    return None;
                }
                Some(Command::NEW {
                    body: input[1..].join(" "),
                })
            }
            "delete" => Some(Command::DELETE),
            "read" => Some(Command::READ),
            _ => Some(Command::NEW {
                body: input.join(" "),
            }),
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

                // Persist tracked state in CWD.
                storage::tracked::set_tracked_file_path(created_path.to_string_lossy().as_ref())
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                println!("Initialized empty record: {}", created_path.display());
                Ok(())
            }

            Command::TRACK { name_of_log } => {
                // Reuse init filename normalization rules (adds `.bif` if missing;
                // rejects path separators).
                let file_name =
                    domain::log_filename::normalize_log_filename(Some(name_of_log.as_str()))?;

                let path = storage::tracked::resolve_existing_bif_in_cwd(&file_name)
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                storage::tracked::set_tracked_file_path(path.to_string_lossy().as_ref())
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                println!("Tracked record: {}", path.display());
                Ok(())
            }

            Command::NEW { body } => {
                let tracked = require_tracked_log()?;

                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|err| CliError::InvalidArgs {
                        message: format!("clock error: {err}"),
                    })?
                    .as_secs()
                    .to_string();

                let stamp =
                    domain::entry::Stamp::new(timestamp, domain::entry::EntryLevel::INFO, None);

                let entry = domain::entry::Entry::new(stamp, body.clone());
                let record = entry.to_record();

                storage::fs_store::append_record_line(std::path::Path::new(&tracked), &record)
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                Ok(())
            }

            Command::DELETE => {
                let tracked = require_tracked_log()?;

                storage::fs_store::delete_last_record_line(std::path::Path::new(&tracked))
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                Ok(())
            }

            Command::READ => {
                let tracked = require_tracked_log()?;

                let contents = std::fs::read_to_string(&tracked)
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                print!("{contents}");
                Ok(())
            }
        }
    }
}
