// File system storage implementation.
//
// Currently contains only `init`-related helpers.

use std::env;
use std::fs::{self, OpenOptions};
use std::io;
use std::io::Write;
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

/// Appends a single record line to an existing `.bif` file.
///
/// - Opens in append mode.
/// - Writes `line` followed by `\n`.
pub fn append_record_line(path: &Path, line: &str) -> io::Result<()> {
    if line.contains('\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "record line must not contain newline",
        ));
    }

    let mut file = OpenOptions::new().append(true).open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Removes the last record line from a `.bif` file.
///
/// Current strategy (simple): read whole file, drop last line, rewrite whole file.
pub fn delete_last_record_line(path: &Path) -> io::Result<()> {
    let contents = fs::read_to_string(path)?;

    // If file is empty (or only whitespace), treat as "nothing to delete".
    if contents.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no entries to delete",
        ));
    }

    let mut lines: Vec<&str> = contents.lines().collect();
    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no entries to delete",
        ));
    }

    lines.pop();

    let new_contents = if lines.is_empty() {
        String::new()
    } else {
        // Preserve standard trailing newline for non-empty files.
        format!("{}\n", lines.join("\n"))
    };

    fs::write(path, new_contents)
}

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
