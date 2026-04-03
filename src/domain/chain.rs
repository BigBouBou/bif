// A note chain is a sequence of notes that are linked together
// The app points to the current note in the chain
pub struct Chain {
    pub current_note: PathBuf,
    pub notes: Vec<Note>,
}
