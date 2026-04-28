// LEGACY - to remove

use std::fmt;
use std::path::PathBuf;

/// Application-level errors.
///
/// Keep this small and actionable in CLI output.
#[derive(Debug)]
pub enum AppError {
    /// The command string does not match any known command.
    UnknownCommand { got: String },

    /// CLI arguments are invalid or missing.
    InvalidArgs { message: String },

    /// The user provided a record name that cannot be turned into a safe filename.
    InvalidRecordName { name: String, reason: String },

    /// Tried to create a file that already exists.
    AlreadyExists { path: PathBuf },

    /// Any I/O error that didn't get a more specific mapping.
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::UnknownCommand { got } => write!(f, "unknown command: {got}"),
            AppError::InvalidArgs { message } => write!(f, "invalid arguments: {message}"),
            AppError::InvalidRecordName { name, reason } => {
                write!(f, "invalid record name '{name}': {reason}")
            }
            AppError::AlreadyExists { path } => {
                write!(f, "file already exists: {}", path.display())
            }
            AppError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
