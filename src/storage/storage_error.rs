use std::path::PathBuf;
use thiserror::Error;

/// Errors produced by the storage layer (filesystem, persistence).
///
/// This layer should not know about CLI concepts. Keep errors focused on I/O
/// and persistence concerns, and let higher layers map them to user-facing
/// messages / exit codes.
#[derive(Debug, Error)]
pub enum StorageError {
    /// A raw I/O error coming from the OS/filesystem.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Attempted to create something that already exists.
    #[error("path already exists: {path}")]
    AlreadyExists { path: PathBuf },

    /// Provided path/filename is invalid for the intended storage operation.
    #[error("invalid path: {path}. {reason}")]
    InvalidPath { path: String, reason: String },
}

impl StorageError {
    /// Helper to map a `std::io::Error` into a more specific `StorageError`
    /// when possible.
    pub fn from_io(err: std::io::Error, path: Option<PathBuf>) -> StorageError {
        match err.kind() {
            std::io::ErrorKind::AlreadyExists => StorageError::AlreadyExists {
                path: path.unwrap_or_else(|| PathBuf::from("<unknown>")),
            },
            _ => StorageError::Io(err),
        }
    }
}
