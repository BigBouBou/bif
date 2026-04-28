use thiserror::Error;

/// Errors for the `domain` layer.
///
/// Design goals:
/// - No dependencies on `cli` or `storage` (keep the domain pure).
/// - Represents invariant violations and parsing/validation failures.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid record filename: {name} ({reason})")]
    InvalidRecordName { name: String, reason: String },

    #[error(transparent)]
    EntryParse(#[from] EntryParseError),
}

/// Placeholder error type for entry parsing/validation.
///
/// As the record format stabilizes, you can expand this with more specific variants
/// (invalid stamp format, invalid level, invalid escape sequence, etc.).
#[derive(Debug, Error)]
pub enum EntryParseError {
    #[error("{0}")]
    Message(String),
}

impl EntryParseError {
    pub fn new(message: impl Into<String>) -> Self {
        EntryParseError::Message(message.into())
    }
}
