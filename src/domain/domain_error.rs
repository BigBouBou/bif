use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid record filename: {name} ({reason})")]
    InvalidRecordName { name: String, reason: String },

    #[error(transparent)]
    EntryParse(#[from] EntryParseError),
}

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
