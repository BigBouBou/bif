// Bif (Before I Forget) is a note taking app in the CLI.
// It allows you to write quick notes while doing other tasks directly in the terminal.

fn main() {
    println!("{}", bif::welcome());

    let input: Vec<String> = std::env::args().skip(1).collect();

    // Basic error handling: print the error and exit with a non-zero code.
    //
    // After the error refactor, `bif::run` returns `CliError` (not `AppError`).
    if let Err(err) = bif::run(input) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
