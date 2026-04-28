// Exposes run() and core wiring
pub mod cli;
pub mod domain;
pub mod error;
pub mod storage;

use crate::cli::cli_error::CliError;

pub fn welcome() -> String {
    "Welcome to BIF! A lazy CLI note-taking app.".to_string()
}

pub fn run(args: Vec<String>) -> Result<(), CliError> {
    let cmd = cli::command::Command::parse(&args).ok_or_else(|| CliError::UnknownCommand {
        got: args.get(0).cloned().unwrap_or_default(),
    })?;

    cmd.execute()
}
