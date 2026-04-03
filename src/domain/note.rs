//bif frontmatter

pub struct NoteId {
    pub value: u8,
}

pub struct NoteMeta {
    pub id: NoteId,
    pub is_anchor: bool,
    pub prev: Option<NoteId>,
    pub next: Option<NoteId>,
}

impl NoteMeta {
    pub fn to_frontmatter() -> Self {}
    pub fn from_frontmatter(frontmatter: &str) -> Result<Self, NoteParseError> {}
}

pub struct Note {
    pub header: NoteMeta,
    pub body: String,
}

impl Note {
    pub fn parse(markdown: &str) -> Result<Note, NoteParseError> {}
    pub fn render(&self) -> String {}

    fn split_frontmatter_and_body<'a>(
        markdown: &'a str,
    ) -> Result<(&'a str, &'a str), NoteParseError> {
        // 1. Check if it starts with "---". If not, bail immediately.
        let rest = markdown
            .strip_prefix("---")
            .ok_or(NoteParseError::InvalidFormat)?;

        // 2. Find the closing "---".
        let end_index = rest.find("---").ok_or(NoteParseError::InvalidFormat)?;

        // 3. Slice the strings based on the index we found
        let frontmatter = rest[..end_index].trim();
        let body = rest[end_index + 3..].trim();

        Ok((frontmatter, body))
    }

    fn parse_meta_lines(lines: &[&str]) -> Result<NoteMeta, NoteParseError> {}
}
