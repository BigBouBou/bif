use std::fs;
use std::path::Path;

use crate::cli::command::Command;
use crate::domain::entry::Entry;

fn temp_dir(prefix: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    let unique = format!(
        "{}_{}",
        prefix,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    p.push(unique);
    fs::create_dir_all(&p).unwrap();
    p
}

fn write_global_config(dir: &Path, contents: &str) {
    // On unix-like: $XDG_CONFIG_HOME/bif/config.json
    // On windows the code prefers %APPDATA%, but tests will run in whatever platform;
    // so we set XDG_CONFIG_HOME and rely on that branch.
    let cfg_dir = dir.join("bif");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(cfg_dir.join("config.json"), contents).unwrap();
}

#[test]
fn new_populates_meta_with_cfg_hash_and_provider_outputs() {
    let root = temp_dir("bif_new_meta");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    // Prepare a tracked log.
    let log_path = cwd.join("log.bif");
    fs::write(&log_path, "").unwrap();
    fs::write(
        cwd.join(".bif-tracked"),
        format!("{}\n", log_path.display()),
    )
    .unwrap();

    // Prepare config.
    let xdg = root.join("xdg");
    write_global_config(
        &xdg,
        r#"{
  "new_stamp_ids": ["time", "level", "cwd"],
  "pretty": {
    "meta_keys": [],
    "meta_sep": " ",
    "legacy_stamp_format": {"parts": []}
  }
}"#,
    );

    // Run the command in the prepared cwd.
    std::env::set_current_dir(&cwd).unwrap();
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
    }

    Command::NEW {
        body: "hello".to_string(),
    }
    .execute()
    .unwrap();

    let contents = fs::read_to_string(&log_path).unwrap();
    let line = contents.lines().next().unwrap();
    let e = Entry::from_record(line).unwrap();

    assert_eq!(e.body, "hello");
    assert!(e.meta.contains_key("_cfg_hash"));
    assert_eq!(e.meta.get("level"), Some(&"INFO".to_string()));
    assert_eq!(e.meta.get("cwd"), Some(&cwd.to_string_lossy().to_string()));
    assert!(e.meta.get("time").unwrap().parse::<u64>().is_ok());
}
