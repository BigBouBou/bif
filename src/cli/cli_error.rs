use crate::domain::domain_error::DomainError;
use crate::storage::storage_error::StorageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("unknown command: {got}")]
    UnknownCommand { got: String },

    #[error("invalid arguments: {message}")]
    InvalidArgs { message: String },

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Storage(#[from] StorageError),
}
