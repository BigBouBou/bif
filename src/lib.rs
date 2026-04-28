// Exposes run() and core wiring
pub mod cli;
pub mod domain;
pub mod error;
pub mod storage;

use crate::cli::cli_error::CliError;

/// Returns a small welcome banner.
pub fn welcome() -> String {
    "Welcome to BIF! A lazy CLI note-taking app.".to_string()
}

/// Parses and executes a CLI command.
///
/// Returns a CLI-level error if the command is unknown or fails during execution.
pub fn run(args: Vec<String>) -> Result<(), CliError> {
    let cmd = cli::command::Command::parse(&args).ok_or_else(|| CliError::UnknownCommand {
        got: args.get(0).cloned().unwrap_or_default(),
    })?;

    cmd.execute()
}
