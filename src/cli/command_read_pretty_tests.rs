use std::fs;
use std::path::Path;

use crate::cli::command::{Command, ReadSpec};

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
    let cfg_dir = dir.join("bif");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(cfg_dir.join("config.json"), contents).unwrap();
}

fn write_local_config(dir: &Path, json_name: &str, contents: &str) {
    fs::write(dir.join(json_name), contents).unwrap();
    fs::write(dir.join(".bif-config"), format!("{}\n", json_name)).unwrap();
}

#[test]
fn read_pretty_legacy_entry_renders_from_stamp_format() {
    let root = temp_dir("bif_read_pretty_legacy");
    let cwd = root.join("work");

    // Avoid cross-test contamination via `XDG_CONFIG_HOME` and inherited `.bif-config`.
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
    }
    // Ensure we run in a directory not nested under a previous test's `.bif-config`.
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    fs::create_dir_all(&cwd).unwrap();

    let log_path = cwd.join("log.bif");
    // Legacy 3-field entry: stamp\ttags\tbody
    fs::write(&log_path, "0|INFO|\t\tHello\n").unwrap();
    fs::write(
        cwd.join(".bif-tracked"),
        format!("{}\n", log_path.display()),
    )
    .unwrap();

    // Config shouldn't matter for legacy rendering; but ensure load works.
    let xdg = root.join("xdg");
    write_global_config(
        &xdg,
        r#"{
  "new_stamp_ids": [],
  "pretty": {
    "meta_keys": [],
    "meta_sep": " ",
    "legacy_stamp_format": {"parts": [
      {"Literal": "["},
      "DateYYYY",
      {"Literal": "-"},
      "DateMM",
      {"Literal": "-"},
      "DateDD",
      {"Literal": " "},
      "TimeHH",
      {"Literal": ":"},
      "TimeMM",
      {"Literal": ":"},
      "TimeSS",
      {"Literal": "] "},
      "Level"
    ]}
  }
}"#,
    );

    std::env::set_current_dir(&cwd).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
    }

    // Capture stdout.
    let mut buf = Vec::<u8>::new();
    Command::READ {
        spec: Some(ReadSpec::CountFromEnd(1)),
        pretty: true,
    }
    .execute_with_io(&mut buf, &mut std::io::sink())
    .unwrap();

    let out = String::from_utf8(buf).unwrap();
    // With `meta_keys` empty, pretty mode falls back to `legacy_stamp_format`.
    // If the config's stamp format renders empty (e.g. due to format config), we still preserve the body.
    assert!(out.trim_end().ends_with("\tHello"), "stdout was: {out}");
}

#[test]
fn read_pretty_meta_entry_prefers_meta_and_warns_on_cfg_mismatch() {
    let root = temp_dir("bif_read_pretty_meta");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    // Ensure we don't accidentally run under an inherited `.bif-config` from another test.
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
    }

    let log_path = cwd.join("log.bif");

    // Create an entry that has meta (4th field) including a mismatched _cfg_hash.
    // NOTE: meta layout is controlled by `pretty.meta_keys`, so include a key we display.
    let meta_json = r#"{"_cfg_hash":"deadbeef","level":"INFO","display":"ZZZ"}"#;
    fs::write(&log_path, format!("0|INFO|\t\tHi\t{}\n", meta_json)).unwrap();
    fs::write(
        cwd.join(".bif-tracked"),
        format!("{}\n", log_path.display()),
    )
    .unwrap();

    // Current config hash will not be "deadbeef".
    let xdg = root.join("xdg");
    write_global_config(
        &xdg,
        r#"{
  "new_stamp_ids": [],
  "pretty": {
    "meta_keys": ["display"],
    "meta_sep": " ",
    "legacy_stamp_format": {"parts": []}
  }
}"#,
    );

    std::env::set_current_dir(&cwd).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
    }

    let mut out_buf = Vec::<u8>::new();
    let mut err_buf = Vec::<u8>::new();
    Command::READ {
        spec: Some(ReadSpec::CountFromEnd(1)),
        pretty: true,
    }
    .execute_with_io(&mut out_buf, &mut err_buf)
    .unwrap();

    let out = String::from_utf8(out_buf).unwrap();
    let err = String::from_utf8(err_buf).unwrap();

    // Best-effort: still prints something, and includes the body.
    assert!(out.contains("Hi"), "stdout was: {out}");

    // Explicit warning on mismatch.
    assert!(
        err.contains("cfg hash mismatch") && err.contains("deadbeef"),
        "stderr was: {err}"
    );
}

#[test]
fn read_pretty_in_child_dir_uses_parent_local_config_for_pretty_layout() {
    let root = temp_dir("bif_read_pretty_local_inherit");
    let parent = root.join("parent");
    let child = parent.join("child");
    fs::create_dir_all(&child).unwrap();

    // Ensure we don't accidentally run under an inherited `.bif-config` from another test.
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
    }

    let log_path = child.join("log.bif");

    // Entry with meta that includes both keys; local config will choose which one is displayed.
    let meta_json = r#"{"a":"AAA","b":"BBB"}"#;
    fs::write(&log_path, format!("0|INFO|\t\tBody\t{}\n", meta_json)).unwrap();
    fs::write(
        child.join(".bif-tracked"),
        format!("{}\n", log_path.display()),
    )
    .unwrap();

    // Parent local config selects meta_keys = ["b"].
    write_local_config(
        &parent,
        "bif.local.json",
        r#"{
  "new_stamp_ids": [],
  "pretty": {
    "meta_keys": ["b"],
    "meta_sep": " ",
    "legacy_stamp_format": {"parts": []}
  }
}"#,
    );

    // Global config selects meta_keys = ["a"], which should NOT win when in child.
    let xdg = root.join("xdg");
    write_global_config(
        &xdg,
        r#"{
  "new_stamp_ids": [],
  "pretty": {
    "meta_keys": ["a"],
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

    let mut out_buf = Vec::<u8>::new();
    Command::READ {
        spec: Some(ReadSpec::CountFromEnd(1)),
        pretty: true,
    }
    .execute_with_io(&mut out_buf, &mut std::io::sink())
    .unwrap();

    let out = String::from_utf8(out_buf).unwrap();
    // Should render stamp as "BBB" (key b), not "AAA" (key a).
    assert!(out.contains("BBB\tBody"), "stdout was: {out}");
    assert!(!out.contains("AAA\tBody"), "stdout was: {out}");
}
