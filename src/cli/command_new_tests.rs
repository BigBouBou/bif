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

fn write_local_config(dir: &Path, json_name: &str, contents: &str) {
    fs::write(dir.join(json_name), contents).unwrap();
    fs::write(dir.join(".bif-config"), format!("{}\n", json_name)).unwrap();
}

#[test]
fn new_populates_meta_with_cfg_hash_and_provider_outputs() {
    let root = temp_dir("bif_new_meta");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    // Ensure we don't accidentally run under an inherited `.bif-config` from another test.
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
    }

    // Prepare a tracked log.
    let log_path = cwd.join("log.bif");
    fs::write(&log_path, "").unwrap();
    fs::write(cwd.join(".bif-tracked"), "log.bif\n").unwrap();

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
        std::env::remove_var("XDG_CONFIG_HOME");
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

#[test]
fn new_in_child_dir_uses_parent_local_config_for_provider_selection() {
    let root = temp_dir("bif_new_local_inherit");
    let parent = root.join("parent");
    let child = parent.join("child");
    fs::create_dir_all(&child).unwrap();

    // Ensure we don't accidentally run under an inherited `.bif-config` from another test.
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
    }

    // Prepare a tracked log in the child directory.
    let log_path = child.join("log.bif");
    fs::write(&log_path, "").unwrap();
    fs::write(
        child.join(".bif-tracked"),
        format!("{}\n", log_path.display()),
    )
    .unwrap();

    // Write a local config in the parent that should be inherited by the child.
    write_local_config(
        &parent,
        "bif.local.json",
        r#"{
  "new_stamp_ids": ["cwd"],
  "pretty": {
    "meta_keys": [],
    "meta_sep": " ",
    "legacy_stamp_format": {"parts": []}
  }
}"#,
    );

    // Also set a global config that would differ, to prove we didn't use it.
    let xdg = root.join("xdg");
    write_global_config(
        &xdg,
        r#"{
  "new_stamp_ids": ["level"],
  "pretty": {
    "meta_keys": [],
    "meta_sep": " ",
    "legacy_stamp_format": {"parts": []}
  }
}"#,
    );

    std::env::set_current_dir(&child).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
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

    // From parent-local config: includes "cwd".
    assert_eq!(
        e.meta.get("cwd"),
        Some(&child.to_string_lossy().to_string())
    );
    // Not from global config.
    assert!(!e.meta.contains_key("level"));
    assert!(e.meta.contains_key("_cfg_hash"));
}
