// Help message rendering

pub fn render() {
    // AI generated preview
    println!("Usage:");
    println!("  bif <command> [options]");
    println!();
    println!("Commands:");
    println!("  init    Initialize a new BIF chain, and track its anchor file");
    println!("  new     Create and track a new BIF file");
    println!("  append  Append an entryto the tracked BIF file");
    println!("  delete  Delete the tracked BIF file");
    println!("  read    Read the tracked BIF note file");
}
