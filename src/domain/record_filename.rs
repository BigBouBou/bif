use crate::domain::domain_error::DomainError;

/// Normalizes the record filename for `bif init`.
///
/// Rules:
/// - No name => `Record.bif`
/// - Empty after trim => error
/// - path separators => error
/// - `name` without `.bif` => append `.bif`
pub fn normalize_record_filename(name: Option<&str>) -> Result<String, DomainError> {
    let default_name = "record.bif";

    let raw = match name {
        None => return Ok(default_name.to_string()),
        Some(n) => n.trim(),
    };

    if raw.is_empty() {
        return Err(DomainError::InvalidRecordName {
            name: raw.to_string(),
            reason: "record name cannot be empty".to_string(),
        });
    }

    if raw.contains('/') || raw.contains('\\') {
        return Err(DomainError::InvalidRecordName {
            name: raw.to_string(),
            reason: "record name must not contain path separators".to_string(),
        });
    }

    if raw.ends_with(".bif") {
        Ok(raw.to_string())
    } else {
        Ok(format!("{raw}.bif"))
    }
}
