// Bif (Before I Forget) is a note taking app in the CLI.
// It allows you to write quick notes while doing other tasks directly in the terminal.

pub mod cli;
pub mod domain;
pub mod storage;

use crate::cli::command::Command;
use std::env;

fn main() {
    println!("{}", bif::welcome());

    let input: Vec<String> = std::env::args().skip(1).collect();

    println!("DEBUG MAIN: Arguments reçus = {:?}", input);

    bif::run(input);
}
