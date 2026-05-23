use std::fs;

use crate::cli::command::Command;

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

fn write_global_config(dir: &std::path::Path, contents: &str) {
    let cfg_dir = dir.join("bif");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(cfg_dir.join("config.json"), contents).unwrap();
}

#[test]
fn init_with_config_creates_json_and_tracks_it_locally() {
    let root = temp_dir("bif_init_with_config");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    // Ensure we don't run under an inherited `.bif-config` from another test.
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
    }

    // Provide a global config to copy.
    let xdg = root.join("xdg");
    write_global_config(
        &xdg,
        r#"{"new_stamp_ids":["time"],"pretty":{"meta_keys":["a"],"meta_sep":" | ","legacy_stamp_format":{"parts":[]}}}"#,
    );

    std::env::set_current_dir(&cwd).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
        std::env::remove_var("APPDATA");
    }

    // Run init with local config creation.
    // Provide an explicit log name so we don't collide with an existing `log.bif` from other tests.
    Command::parse(&vec![
        "init".to_string(),
        "my.bif".to_string(),
        "--config".to_string(),
        "./mon_config_1.json".to_string(),
    ])
    .unwrap()
    .execute_with_io(&mut std::io::sink(), &mut std::io::sink())
    .unwrap();

    // Config file should be created (create_new semantics; content equals global json).
    let created = fs::read_to_string(cwd.join("mon_config_1.json")).unwrap();
    assert!(created.contains("\"new_stamp_ids\":[\"time\"]"));

    // `.bif-config` should track the relative path exactly as provided.
    let dot_contents = fs::read_to_string(cwd.join(".bif-config")).unwrap();
    assert_eq!(dot_contents, "./mon_config_1.json\n");

    // `config show` should report local now.
    let mut show_out = Vec::<u8>::new();
    Command::CONFIG_SHOW
        .execute_with_io(&mut show_out, &mut std::io::sink())
        .unwrap();
    let show = String::from_utf8(show_out).unwrap();
    assert!(
        show.lines().next().unwrap_or("") == "local",
        "stdout was: {show}"
    );
}

#[test]
fn init_with_config_uses_default_when_global_missing() {
    let root = temp_dir("bif_init_with_config_default");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
    }

    // Point XDG to empty dir so global config doesn't exist.
    let xdg = root.join("xdg");
    fs::create_dir_all(&xdg).unwrap();

    std::env::set_current_dir(&cwd).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
        std::env::remove_var("APPDATA");
    }

    Command::parse(&vec![
        "init".to_string(),
        "my.bif".to_string(),
        "--config".to_string(),
        "bif.local.json".to_string(),
    ])
    .unwrap()
    .execute_with_io(&mut std::io::sink(), &mut std::io::sink())
    .unwrap();

    let created = fs::read_to_string(cwd.join("bif.local.json")).unwrap();
    // Default has empty new_stamp_ids.
    assert!(created.contains("\"new_stamp_ids\":["));
}
