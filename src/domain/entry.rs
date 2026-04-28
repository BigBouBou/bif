pub struct EntryIndexFromEnd(pub usize);

pub struct Entry {
    pub stamp: Stamp,
    pub body: String,
    pub tags: Vec<String>,
}

impl Entry {
    /// Creates a new `Entry` with the given `stamp` and body `text`.
    pub fn new(stamp: Stamp, text: String) -> Entry {
        Entry {
            stamp,
            body: text,
            tags: Vec::new(),
        }
    }

    /// INVARIANTS:
    /// - `stamp` is valid (`Stamp::validate`)
    /// - `body` must be non-empty after trimming
    /// - Each tag must be non-empty after trimming and must not contain `,`
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

    /// Converts this entry to a single-line record string.
    ///
    /// Record format:
    ///
    /// <STAMP>\t<BODY>\t<TAG1,TAG2,...>
    ///
    /// - `\t` as delimiter
    /// - `BODY` uses a very small escaping scheme so the record stays one line:
    ///   `\` => `\\`, tab => `\t`, newline => `\n`, carriage return => `\r`.
    /// - Tags are stored as comma-separated values.
    pub fn to_record(&self) -> String {
        let stamp = self.stamp.to_record();
        let body = escape_field(&self.body); // REVIEW - Is it necessary to escape the body?
        let tags = if self.tags.is_empty() {
            String::new()
        } else {
            self.tags.join(",")
        };

        format!("{stamp}\t{body}\t{tags}")
    }

    /// Parses a record string into an `Entry`.
    ///
    /// 1. Split on `\t` into 3 fields: stamp, body, tags.
    /// 2. Parse the stamp with `Stamp::from_record`.
    /// 3. Unescape the body using the same escaping scheme as `to_record`.
    /// 4. Split tags on `,` (empty means "no tags").
    /// 5. Validate the constructed entry to enforce invariants.
    pub fn from_record(line: &str) -> Result<Entry, EntryParseError> {
        let mut parts = line.splitn(3, '\t');
        let stamp_part = parts.next().unwrap_or("");
        let body_part = parts.next().ok_or(EntryParseError::InvalidEntryFormat {
            expected: "STAMP<TAB>BODY<TAB>TAGS",
            got: line.to_string(),
        })?;
        let tags_part = parts.next().ok_or(EntryParseError::InvalidEntryFormat {
            expected: "STAMP<TAB>BODY<TAB>TAGS",
            got: line.to_string(),
        })?; // REVIEW - tag not necessary?

        if stamp_part.is_empty() {
            return Err(EntryParseError::InvalidEntryFormat {
                expected: "non-empty STAMP",
                got: line.to_string(),
            });
        }

        let stamp = Stamp::from_record(stamp_part)?;
        let body = unescape_field(body_part).map_err(|reason| EntryParseError::InvalidEscape {
            field: "body",
            reason,
        })?;

        let tags: Vec<String> = if tags_part.is_empty() {
            Vec::new()
        } else {
            tags_part.split(',').map(|s| s.to_string()).collect()
        };

        let entry = Entry { stamp, body, tags };
        entry.validate()?;
        Ok(entry)
    }
}

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
            //REVIEW - Should be string?
            if source.trim().is_empty() {
                return Err(EntryParseError::EmptySource);
            }
            if source.contains('|') {
                return Err(EntryParseError::InvalidSourceContainsDelimiter {
                    source: source.clone(),
                });
            }
        }

        Ok(())
    }

    /// Parses a stamp record string into a `Stamp`.
    ///
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

pub enum EntryParseError {
    InvalidEntryFormat { expected: &'static str, got: String },
    InvalidStampFormat { expected: &'static str, got: String },

    EmptyTimestamp,
    EmptySource,
    EmptyBody,
    EmptyTag,

    InvalidLevel { got: String },

    InvalidSourceContainsDelimiter { source: String },
    InvalidTagContainsComma { tag: String },

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
