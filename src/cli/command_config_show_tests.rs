use std::fs;
use std::path::Path;

use crate::cli::command::Command;
use crate::cli::config::GlobalConfig;

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
fn parse_config_show() {
    let cmd = Command::parse(&vec!["config".to_string(), "show".to_string()]).unwrap();
    assert!(matches!(cmd, Command::CONFIG_SHOW));
}

#[test]
fn config_show_reports_local_origin_with_paths() {
    let root = temp_dir("bif_config_show_local");
    let parent = root.join("parent");
    let child = parent.join("child");
    fs::create_dir_all(&child).unwrap();

    // Local config in parent.
    write_local_config(
        &parent,
        "bif.local.json",
        r#"{"new_stamp_ids":[],"pretty":{"meta_keys":[]}}"#,
    );

    // Ensure a global config exists but shouldn't win.
    let xdg = root.join("xdg");
    write_global_config(
        &xdg,
        r#"{"new_stamp_ids":["time"],"pretty":{"meta_keys":[]}}"#,
    );

    std::env::set_current_dir(&child).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
        std::env::remove_var("APPDATA");
    }

    let mut out_buf = Vec::<u8>::new();
    Command::CONFIG_SHOW
        .execute_with_io(&mut out_buf, &mut std::io::sink())
        .unwrap();

    let out = String::from_utf8(out_buf).unwrap();
    let dot = parent.join(".bif-config");
    let json = parent.join("bif.local.json").canonicalize().unwrap();

    assert!(
        out.lines().next().unwrap_or("") == "local",
        "stdout was: {out}"
    );
    assert!(
        out.contains(&format!(".bif-config: {}", dot.display())),
        "stdout was: {out}"
    );
    assert!(
        out.contains(&format!("config: {}", json.display())),
        "stdout was: {out}"
    );
}

#[test]
fn config_show_reports_default_when_no_global_config_file_exists() {
    let root = temp_dir("bif_config_show_default");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    let xdg = root.join("xdg");
    fs::create_dir_all(&xdg).unwrap();

    std::env::set_current_dir(&cwd).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
        std::env::remove_var("APPDATA");
    }

    let mut out_buf = Vec::<u8>::new();
    Command::CONFIG_SHOW
        .execute_with_io(&mut out_buf, &mut std::io::sink())
        .unwrap();

    let out = String::from_utf8(out_buf).unwrap();
    assert!(out.contains("default"), "stdout was: {out}");
    assert!(out.contains("config: default"), "stdout was: {out}");
}

#[test]
fn config_show_reports_global_with_path_when_global_config_file_exists() {
    let root = temp_dir("bif_config_show_global");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    let xdg = root.join("xdg");
    write_global_config(
        &xdg,
        r#"{"new_stamp_ids":["time"],"pretty":{"meta_keys":[]}}"#,
    );

    std::env::set_current_dir(&cwd).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
        std::env::remove_var("APPDATA");
    }

    let mut out_buf = Vec::<u8>::new();
    Command::CONFIG_SHOW
        .execute_with_io(&mut out_buf, &mut std::io::sink())
        .unwrap();

    let out = String::from_utf8(out_buf).unwrap();
    let global_path = crate::cli::config::default_config_path().unwrap();

    assert!(
        out.lines().next().unwrap_or("") == "global",
        "stdout was: {out}"
    );
    assert!(
        out.contains(&format!("config: {}", global_path.display())),
        "stdout was: {out}"
    );

    // Keep global config for subsequent tests in this module.
    let _ = GlobalConfig::load_global().unwrap();
}
