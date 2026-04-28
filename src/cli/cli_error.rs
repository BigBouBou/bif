use crate::domain::domain_error::DomainError;
use crate::error::AppError;
use crate::storage::storage_error::StorageError;
use thiserror::Error;

/// Top-level error for the CLI/application boundary.
///
/// This is the error type you should return from `bif::run(...)` and ultimately
/// print in `main`.
///
/// Layering:
/// - `cli` can depend on `domain` and `storage`
/// - `domain` should not depend on `cli` or `storage`
/// - `storage` should not depend on `cli`
///
/// `AppError` is kept as a legacy/error-mapping type for now; over time you can
/// migrate call sites to return `DomainError` / `StorageError` directly and drop
/// `AppError` altogether.
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

    #[error(transparent)]
    App(#[from] AppError),
}
