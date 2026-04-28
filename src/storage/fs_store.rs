// File system storage implementation.
//
// Currently contains only `init`-related helpers.

use std::env;
use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

/// Creates an empty `.bif` record file in the current working directory.
///
/// - `file_name` must be a single file name (no directory separators).
/// - Fails if the file already exists.
/// - Returns the created path on success.
pub fn create_empty_record_file_in_cwd(file_name: &str) -> io::Result<PathBuf> {
    validate_file_name(file_name)?;

    let cwd = env::current_dir()?;
    let path = cwd.join(file_name);

    // `create_new(true)` ensures we never overwrite an existing record by accident.
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map(|_file| path)
}

/// Validates that `file_name` is safe to create in the current directory.
fn validate_file_name(file_name: &str) -> io::Result<()> {
    if file_name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "record file name cannot be empty",
        ));
    }

    // Disallow any path components; `init` must create the file "right where you are".
    if file_name.contains('/') || file_name.contains('\\') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "record file name must not contain path separators",
        ));
    }

    // Extra guard: make sure it is a "plain file name" (no parent directories).
    if Path::new(file_name).file_name().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "record file name is invalid",
        ));
    }

    Ok(())
}
