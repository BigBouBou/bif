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

#[test]
fn config_set_local_creates_dotfile_and_config_show_reports_local() {
    let root = temp_dir("bif_config_set_local_create");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    // Ensure we don't accidentally run under an inherited `.bif-config` from another test.
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
    }

    // Create a target config JSON file.
    fs::write(
        cwd.join("mon_config.json"),
        r#"{"new_stamp_ids":[],"pretty":{"meta_keys":[]}}"#,
    )
    .unwrap();

    std::env::set_current_dir(&cwd).unwrap();
    // Avoid cross-test contamination via `XDG_CONFIG_HOME`.
    // Set it to a unique empty directory so global config resolution can't accidentally
    // find a real config on the developer machine.
    let xdg = root.join("xdg");
    fs::create_dir_all(&xdg).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
        std::env::remove_var("APPDATA");
    }

    // Set local tracking.
    let mut out_buf = Vec::<u8>::new();
    Command::parse(&vec![
        "config".to_string(),
        "set".to_string(),
        "./mon_config.json".to_string(),
        "--local".to_string(),
    ])
    .unwrap()
    .execute_with_io(&mut out_buf, &mut std::io::sink())
    .unwrap();

    // `.bif-config` should contain the relative string + newline.
    let dot_contents = fs::read_to_string(cwd.join(".bif-config")).unwrap();
    assert_eq!(dot_contents, "./mon_config.json\n");

    // And `config show` should now report local.
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
fn config_set_local_overwrites_existing_dotfile_and_prints_explicit_message() {
    let root = temp_dir("bif_config_set_local_overwrite");
    let cwd = root.join("work");
    fs::create_dir_all(&cwd).unwrap();

    // Ensure we don't accidentally run under an inherited `.bif-config` from another test.
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("APPDATA");
    }

    fs::write(
        cwd.join("a.json"),
        r#"{"new_stamp_ids":[],"pretty":{"meta_keys":[]}}"#,
    )
    .unwrap();
    fs::write(
        cwd.join("b.json"),
        r#"{"new_stamp_ids":[],"pretty":{"meta_keys":[]}}"#,
    )
    .unwrap();

    // Existing `.bif-config`.
    fs::write(cwd.join(".bif-config"), "./a.json\n").unwrap();

    std::env::set_current_dir(&cwd).unwrap();
    // Avoid cross-test contamination via `XDG_CONFIG_HOME`.
    let xdg = root.join("xdg");
    fs::create_dir_all(&xdg).unwrap();
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", xdg.to_string_lossy().to_string());
        std::env::remove_var("APPDATA");
    }

    let mut out_buf = Vec::<u8>::new();
    Command::parse(&vec![
        "config".to_string(),
        "set".to_string(),
        "./b.json".to_string(),
        "--local".to_string(),
    ])
    .unwrap()
    .execute_with_io(&mut out_buf, &mut std::io::sink())
    .unwrap();

    let out = String::from_utf8(out_buf).unwrap();
    assert!(
        out.contains("Updated local config tracking: ./b.json"),
        "stdout was: {out}"
    );

    let dot_contents = fs::read_to_string(cwd.join(".bif-config")).unwrap();
    assert_eq!(dot_contents, "./b.json\n");
}
