use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const TRACKED_DOTFILE: &str = ".bif-tracked";

/// Returns the absolute path to the tracking dotfile in the current working directory.
pub fn tracked_file_path() -> io::Result<PathBuf> {
    let cwd = env::current_dir()?;
    Ok(cwd.join(TRACKED_DOTFILE))
}

/// Persists the currently tracked `.bif` file path into the CWD tracking dotfile.
///
/// Storage format: a single line containing the path as UTF-8.
pub fn set_tracked_file_path(path: &str) -> io::Result<()> {
    if path.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tracked file path cannot be empty",
        ));
    }

    let dotfile = tracked_file_path()?;
    fs::write(dotfile, format!("{path}\n"))
}

/// Reads the tracked `.bif` file path from the CWD tracking dotfile.
///
/// Returns an error if the dotfile does not exist or is invalid.
pub fn get_tracked_file_path() -> io::Result<String> {
    let dotfile = tracked_file_path()?;
    let contents = fs::read_to_string(dotfile)?;
    let path = contents.lines().next().unwrap_or("").trim();

    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "tracked file path is empty",
        ));
    }

    Ok(path.to_string())
}

/// Resolves a `.bif` file in the current working directory and verifies it exists.
///
/// `file_name` must be a plain filename (no path separators).
pub fn resolve_existing_bif_in_cwd(file_name: &str) -> io::Result<PathBuf> {
    validate_plain_file_name(file_name)?;

    if !is_bif_file_name(file_name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tracked file must have .bif extension",
        ));
    }

    let cwd = env::current_dir()?;
    let path = cwd.join(file_name);

    let meta = fs::metadata(&path)?;
    if !meta.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tracked path is not a file",
        ));
    }

    Ok(path)
}

fn validate_plain_file_name(file_name: &str) -> io::Result<()> {
    if file_name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "record file name cannot be empty",
        ));
    }

    // Disallow any path components; tracking is CWD-first.
    if file_name.contains('/') || file_name.contains('\\') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "record file name must not contain path separators",
        ));
    }

    if Path::new(file_name).file_name().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "record file name is invalid",
        ));
    }

    Ok(())
}

fn is_bif_file_name(file_name: &str) -> bool {
    Path::new(file_name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("bif"))
}
