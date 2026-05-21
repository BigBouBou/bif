use std::collections::BTreeMap;

pub struct EntryIndexFromEnd(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub stamp: Stamp,
    pub body: String,
    pub tags: Vec<String>,
    pub meta: BTreeMap<String, String>,
}

impl Entry {
    pub fn new(stamp: Stamp, text: String) -> Entry {
        Entry {
            stamp,
            body: text,
            tags: Vec::new(),
            meta: BTreeMap::new(),
        }
    }

    /// INVARIANTS:
    /// - `stamp` is valid (`Stamp::validate`)
    /// - `body` must be non-empty after trimming
    /// - tags must not be empty/whitespace and must not contain `,`
    pub fn validate(&self) -> Result<(), EntryParseError> {
        self.stamp.validate()?;

        if self.body.trim().is_empty() {
            return Err(EntryParseError::EmptyBody);
        }

        for tag in &self.tags {
            if tag.trim().is_empty() {
                return Err(EntryParseError::EmptyTag);
            }
            if tag.contains(',') {
                return Err(EntryParseError::InvalidTagContainsComma { tag: tag.clone() });
            }
        }

        Ok(())
    }

    /// Converts this entry to a record string one-liner.
    ///
    /// Record format:
    ///
    /// Legacy record format (no meta):
    ///
    /// <STAMP>\t<TAGS>\t<BODY>
    ///
    /// Extended record format (optional meta as 4th field):
    ///
    /// <STAMP>\t<TAGS>\t<BODY>\t<META_JSON>
    ///
    /// - `\t` as delimiter
    /// - `BODY` uses a very small escaping scheme so the record stays one line:
    ///   `\` => `\\`, tab => `\\t`, newline => `\\n`, carriage return => `\\r`.
    /// - Tags are stored as comma-separated values.
    /// - `META_JSON` is JSON (object mapping string->string) and then escaped using the same field
    ///   escaping scheme as `BODY`.
    pub fn to_record(&self) -> String {
        let stamp = self.stamp.to_record();
        let tags = if self.tags.is_empty() {
            String::new()
        } else {
            self.tags.join(",")
        };
        let body = escape_field(&self.body);

        if self.meta.is_empty() {
            format!("{stamp}\t{tags}\t{body}")
        } else {
            let meta_json = serde_json::to_string(&self.meta)
                .expect("meta map must always be JSON-serializable");
            let meta_field = escape_field(&meta_json);
            format!("{stamp}\t{tags}\t{body}\t{meta_field}")
        }
    }

    /// Parses a record string into an `Entry`.
    ///
    /// 1. Split on `\t` into 3 or 4 fields: stamp, tags, body, (optional) meta.
    /// 2. Parse the stamp with `Stamp::from_record`.
    /// 3. Unescape the body using the same escaping scheme as `to_record`.
    /// 4. Split tags on `,` (empty means "no tags").
    /// 5. If present, unescape meta and parse it as JSON into `BTreeMap<String,String>`.
    /// 6. Validate the constructed entry to enforce invariants.
    pub fn from_record(line: &str) -> Result<Entry, EntryParseError> {
        // Backward-compatible parsing:
        // - legacy:   STAMP<TAB>TAGS<TAB>BODY
        // - extended: STAMP<TAB>TAGS<TAB>BODY<TAB>META_JSON
        let mut parts = line.splitn(4, '\t');
        let stamp_part = parts.next().unwrap_or("");
        let tags_part = parts.next().ok_or(EntryParseError::InvalidEntryFormat {
            expected: "STAMP<TAB>TAGS<TAB>BODY[<TAB>META_JSON]",
            got: line.to_string(),
        })?;
        let body_part = parts.next().ok_or(EntryParseError::InvalidEntryFormat {
            expected: "STAMP<TAB>TAGS<TAB>BODY[<TAB>META_JSON]",
            got: line.to_string(),
        })?;
        let meta_part = parts.next();

        if stamp_part.is_empty() {
            return Err(EntryParseError::InvalidEntryFormat {
                expected: "non-empty STAMP",
                got: line.to_string(),
            });
        }
        let stamp = Stamp::from_record(stamp_part)?;

        let tags: Vec<String> = if tags_part.is_empty() {
            Vec::new()
        } else {
            tags_part.split(',').map(|s| s.to_string()).collect()
        };

        let body = unescape_field(body_part).map_err(|reason| EntryParseError::InvalidEscape {
            field: "body",
            reason,
        })?;

        let meta: BTreeMap<String, String> = match meta_part {
            None => BTreeMap::new(),
            Some(raw_meta) => {
                let meta_json =
                    unescape_field(raw_meta).map_err(|reason| EntryParseError::InvalidEscape {
                        field: "meta",
                        reason,
                    })?;

                serde_json::from_str(&meta_json).map_err(|err| {
                    EntryParseError::InvalidMetaJson {
                        got: meta_json,
                        reason: err.to_string(),
                    }
                })?
            }
        };

        let entry = Entry {
            stamp,
            body,
            tags,
            meta,
        };
        entry.validate()?;
        Ok(entry)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub timestamp: String,
    pub level: EntryLevel,
    pub source: Option<String>,
}

impl Stamp {
    pub fn new(timestamp: String, level: EntryLevel, source: Option<String>) -> Stamp {
        Stamp {
            timestamp,
            level,
            source,
        }
    }

    /// INVARIANTS:
    /// - `timestamp` must be non-empty.
    /// - `source`, if present, must be non-empty after trimming.
    /// - `source` must not contain `|` because `Stamp` uses `|` as a delimiter for its record format.
    pub fn validate(&self) -> Result<(), EntryParseError> {
        if self.timestamp.trim().is_empty() {
            return Err(EntryParseError::EmptyTimestamp);
        }

        if let Some(source) = &self.source {
            if source.trim().is_empty() {
                return Err(EntryParseError::EmptySource);
            }
            if source.contains('|') {
                return Err(EntryParseError::InvalidSourceContainsDelimiter);
            }
        }

        Ok(())
    }

    /// Stamp record format:
    ///
    /// <TIMESTAMP>|<LEVEL>|<SOURCE?>
    ///
    /// - <SOURCE?> may be empty (meaning `None`).
    pub fn from_record(line: &str) -> Result<Stamp, EntryParseError> {
        let mut parts = line.splitn(3, '|');
        let ts = parts.next().unwrap_or("");
        let lvl = parts.next().ok_or(EntryParseError::InvalidStampFormat {
            expected: "TIMESTAMP|LEVEL|SOURCE?",
            got: line.to_string(),
        })?;
        let src = parts.next().ok_or(EntryParseError::InvalidStampFormat {
            expected: "TIMESTAMP|LEVEL|SOURCE?",
            got: line.to_string(),
        })?;

        let level = EntryLevel::from_str(lvl).ok_or_else(|| EntryParseError::InvalidLevel {
            got: lvl.to_string(),
        })?;

        let source = if src.is_empty() {
            None
        } else {
            Some(src.to_string())
        };

        let stamp = Stamp {
            timestamp: ts.to_string(),
            level,
            source,
        };
        stamp.validate()?;
        Ok(stamp)
    }

    /// Converts this `Stamp` to a stamp record string.
    pub fn to_record(&self) -> String {
        let src = self.source.as_deref().unwrap_or("");
        format!("{}|{}|{}", self.timestamp, self.level.to_record(), src)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl EntryLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "DEBUG" => Some(EntryLevel::DEBUG),
            "INFO" => Some(EntryLevel::INFO),
            "WARN" => Some(EntryLevel::WARN),
            "ERROR" => Some(EntryLevel::ERROR),
            _ => None,
        }
    }

    fn to_record(&self) -> &'static str {
        match self {
            EntryLevel::DEBUG => "DEBUG",
            EntryLevel::INFO => "INFO",
            EntryLevel::WARN => "WARN",
            EntryLevel::ERROR => "ERROR",
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EntryParseError {
    #[error("invalid entry format: expected {expected}, got: {got}")]
    InvalidEntryFormat { expected: &'static str, got: String },

    #[error("invalid meta JSON: {reason}; got: {got}")]
    InvalidMetaJson { got: String, reason: String },

    #[error("invalid stamp format: expected {expected}, got: {got}")]
    InvalidStampFormat { expected: &'static str, got: String },

    #[error("empty timestamp")]
    EmptyTimestamp,
    #[error("empty source")]
    EmptySource,
    #[error("empty body")]
    EmptyBody,
    #[error("empty tag")]
    EmptyTag,

    #[error("invalid level: {got}")]
    InvalidLevel { got: String },

    #[error("invalid source contains delimiter '|'")]
    InvalidSourceContainsDelimiter,

    #[error("invalid tag contains comma: {tag}")]
    InvalidTagContainsComma { tag: String },

    #[error("invalid escape in {field}: {reason}")]
    InvalidEscape { field: &'static str, reason: String },
}

/// Escapes a field so it can be stored in a single-line record.
///
/// Implementation details:
/// - This is deliberately minimal and reversible.
/// - We escape backslash first so we don't double-process sequences we introduce.
fn escape_field(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

fn unescape_field(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();

    while let Some(ch) = it.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        let esc = it
            .next()
            .ok_or_else(|| "trailing backslash in escape sequence".to_string())?;
        match esc {
            '\\' => out.push('\\'),
            't' => out.push('\t'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            other => {
                return Err(format!("unknown escape sequence: \\{other}"));
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp() -> Stamp {
        Stamp::new("123".to_string(), EntryLevel::INFO, None)
    }

    #[test]
    fn legacy_roundtrip_unchanged() {
        let mut e = Entry::new(stamp(), "hello\\tworld".to_string());
        e.tags = vec!["a".to_string(), "b".to_string()];

        let record = e.to_record();
        assert!(!record.contains("\t{"));

        let parsed = Entry::from_record(&record).unwrap();
        assert_eq!(parsed.to_record(), record);
        assert!(parsed.meta.is_empty());
    }

    #[test]
    fn meta_roundtrip() {
        let mut e = Entry::new(stamp(), "body".to_string());
        e.meta.insert("k".to_string(), "v".to_string());

        let record = e.to_record();
        let parsed = Entry::from_record(&record).unwrap();

        assert_eq!(parsed.meta.get("k"), Some(&"v".to_string()));
        assert_eq!(parsed.to_record(), record);
    }

    #[test]
    fn invalid_json_error() {
        let line = format!(
            "{}\t\t{}\t{}",
            stamp().to_record(),
            escape_field("b"),
            escape_field("{not json")
        );
        let err = Entry::from_record(&line).unwrap_err();
        match err {
            EntryParseError::InvalidMetaJson { .. } => {}
            other => panic!("expected InvalidMetaJson, got: {other:?}"),
        }
    }
}
