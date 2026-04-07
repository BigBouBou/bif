// Help message rendering

pub fn render() {
    // AI generated preview
    println!("Usage:");
    println!("  bif <command> [options]");
    println!();
    println!("Commands:");
    println!("  init    Initialize a new .bif file, and begins tracking it");
    println!("  track   Track an existing .bif file");
    println!("  new     Create a new entry in the tracked .bif file");
    println!("  delete  Delete the last bif entry");
    println!("  read    Read the tracked .bif note file");
}
