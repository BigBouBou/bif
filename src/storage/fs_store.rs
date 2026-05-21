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
    delete_last_n_record_lines(path, 1)
}

/// Deletes the last N record lines from a `.bif` file.
///
/// - `n == 1` matches `delete_last_record_line`.
/// - Errors if `n` is 0 or if the file has fewer than `n` entries.
pub fn delete_last_n_record_lines(path: &Path, n: usize) -> io::Result<()> {
    if n == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "delete count must be >= 1",
        ));
    }

    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.lines().collect();

    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no entries to delete",
        ));
    }

    if n > lines.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not enough entries to delete",
        ));
    }

    let remaining = &lines[..lines.len() - n];
    write_lines(path, remaining)
}

/// Deletes the Nth record line from the end.
///
/// - `n == 1` => last entry
/// - `n == 2` => second-to-last entry
pub fn delete_record_line_by_index_from_end(path: &Path, n: usize) -> io::Result<()> {
    if n == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "delete index must be >= 1",
        ));
    }

    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.lines().collect();

    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no entries to delete",
        ));
    }

    if n > lines.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "delete index out of range",
        ));
    }

    let idx = lines.len() - n;
    let remaining: Vec<&str> = lines
        .into_iter()
        .enumerate()
        .filter_map(|(i, line)| if i == idx { None } else { Some(line) })
        .collect();

    write_lines(path, &remaining)
}

fn write_lines(path: &Path, lines: &[&str]) -> io::Result<()> {
    let new_contents = if lines.is_empty() {
        String::new()
    } else {
        // Preserve standard trailing newline for non-empty files.
        format!("{}\n", lines.join("\n"))
    };

    fs::write(path, new_contents)
}

/// Reads the last N record lines from a `.bif` file.
///
/// - `n == 1` => last entry only
/// - Errors if `n` is 0 or if the file has fewer than `n` entries.
pub fn read_last_n_record_lines(path: &Path, n: usize) -> io::Result<Vec<String>> {
    if n == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "read count must be >= 1",
        ));
    }

    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.lines().collect();

    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no entries to read",
        ));
    }

    if n > lines.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not enough entries to read",
        ));
    }

    Ok(lines[lines.len() - n..]
        .iter()
        .map(|s| s.to_string())
        .collect())
}

/// Reads the Nth record line from the end.
///
/// - `n == 1` => last entry
/// - `n == 2` => second-to-last entry
pub fn read_record_line_by_index_from_end(path: &Path, n: usize) -> io::Result<String> {
    if n == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "read index must be >= 1",
        ));
    }

    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.lines().collect();

    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no entries to read",
        ));
    }

    if n > lines.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "read index out of range",
        ));
    }

    Ok(lines[lines.len() - n].to_string())
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
