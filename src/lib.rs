//Exposes run() and core wiring

pub fn welcome() -> String {
    "Welcome to BIF! A lazy CLI note-taking app.".to_string()
}

pub fn run(args: Vec<String>) {
    if let Some(cmd) = cli::command::Command::parse(&args) {
        cmd.execute();
    } else {
        eprintln!("Commande inconnue : {}", args[0]);
    }
}

//--------------------//
