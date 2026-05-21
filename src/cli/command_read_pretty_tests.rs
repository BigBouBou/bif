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

#[test]
fn read_pretty_legacy_entry_renders_from_stamp_format() {
    let root = temp_dir("bif_read_pretty_legacy");
    let cwd = root.join("work");
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
  },
  "pretty_stamp_format": {"parts": [
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
}"#,
    );

    std::env::set_current_dir(&cwd).unwrap();
    unsafe {
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
    assert_eq!(out.trim_end(), "[1970-01-01 00:00:00] INFO\tHello");
}

#[test]
fn read_pretty_meta_entry_prefers_meta_and_warns_on_cfg_mismatch() {
    let root = temp_dir("bif_read_pretty_meta");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    let log_path = cwd.join("log.bif");

    // Create an entry that has meta (4th field) including a mismatched _cfg_hash.
    // Meta JSON: {"_cfg_hash":"deadbeef", "level":"INFO"}
    // Field escaping is handled by Entry::from_record, but here we ensure no tabs/newlines.
    // IMPORTANT: the on-disk meta field is escaped with the same scheme as body.
    // In particular, quotes do NOT need escaping; backslashes are treated as escapes.
    let meta_json = r#"{"_cfg_hash":"deadbeef","level":"INFO"}"#;
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
    "meta_keys": ["level"],
    "meta_sep": " ",
    "legacy_stamp_format": {"parts": []}
  }
}"#,
    );

    std::env::set_current_dir(&cwd).unwrap();
    unsafe {
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
    assert!(out.contains("\tHi"));

    // Explicit warning on mismatch.
    assert!(
        err.contains("cfg hash mismatch") && err.contains("deadbeef"),
        "stderr was: {err}"
    );
}
