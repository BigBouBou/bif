use crate::cli::config::GlobalConfig;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Where the effective config was sourced from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigOrigin {
    /// A `.bif-config` in (or above) the current working directory resolved to this JSON file.
    Local {
        /// Path to the discovered `.bif-config` file.
        dotfile_path: PathBuf,
        /// Path to the referenced JSON file (absolute).
        json_path: PathBuf,
    },
    /// The user-level global config location.
    Global,
    /// No config file existed; using defaults.
    Default,
}

/// The resolved configuration used for command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveConfig {
    pub cfg: GlobalConfig,
    pub origin: ConfigOrigin,
}

/// Loads the effective configuration for a given `cwd`.
///
/// Resolution order:
/// 1) Search `cwd`, then parents, for a `.bif-config`.
///    - If found, read it as UTF-8 text.
///    - The file must contain a relative path (relative to the directory containing the `.bif-config`)
///      that points to a JSON config file.
///    - If the referenced JSON is missing or invalid, return a clear error mentioning paths.
/// 2) Fall back to the global config path (see `GlobalConfig::load_global`).
///    - If missing, use defaults.
pub fn load_effective_config(cwd: &Path) -> io::Result<EffectiveConfig> {
    if let Some((dotfile_path, json_path)) = find_local_config_paths(cwd)? {
        let json_str = fs::read_to_string(&json_path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!(
                    "failed to read config JSON at '{}' (referenced by '{}'): {}",
                    json_path.display(),
                    dotfile_path.display(),
                    err
                ),
            )
        })?;

        let cfg: GlobalConfig = serde_json::from_str(&json_str).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid JSON in config file '{}' (referenced by '{}'): {}",
                    json_path.display(),
                    dotfile_path.display(),
                    err
                ),
            )
        })?;

        return Ok(EffectiveConfig {
            cfg,
            origin: ConfigOrigin::Local {
                dotfile_path,
                json_path,
            },
        });
    }

    let cfg = GlobalConfig::load_global()?;
    let origin = if cfg == GlobalConfig::default() {
        // Best-effort classification: if loading global yielded defaults, assume no global file.
        // (If a user explicitly stores the default JSON, this will classify as Default.)
        ConfigOrigin::Default
    } else {
        ConfigOrigin::Global
    };

    Ok(EffectiveConfig { cfg, origin })
}

fn find_local_config_paths(cwd: &Path) -> io::Result<Option<(PathBuf, PathBuf)>> {
    let mut dir = cwd;

    loop {
        let dotfile_path = dir.join(".bif-config");
        if dotfile_path.exists() {
            let raw = fs::read_to_string(&dotfile_path).map_err(|err| {
                io::Error::new(
                    err.kind(),
                    format!(
                        "failed to read .bif-config at '{}': {}",
                        dotfile_path.display(),
                        err
                    ),
                )
            })?;

            let rel = raw.trim();
            if rel.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        ".bif-config at '{}' is empty; expected relative path to a JSON config file",
                        dotfile_path.display()
                    ),
                ));
            }

            if Path::new(rel).is_absolute() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        ".bif-config at '{}' must contain a relative path, got absolute path '{}",
                        dotfile_path.display(),
                        rel
                    ),
                ));
            }

            let base = dir;
            let json_path = base.join(rel);
            let json_path = json_path
                .canonicalize()
                .map_err(|err| io::Error::new(err.kind(), format!(
                    "failed to resolve config JSON path '{}' relative to .bif-config at '{}': {}",
                    json_path.display(),
                    dotfile_path.display(),
                    err
                )))?;

            return Ok(Some((dotfile_path, json_path)));
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_temp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "bif-test-{}-{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        p
    }

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn no_dotfile_uses_global_or_default() {
        let root = unique_temp_dir("no-dotfile");
        fs::create_dir_all(&root).unwrap();

        // Force global config lookup to a temp XDG dir with no config file.
        let xdg = root.join("xdg");
        fs::create_dir_all(&xdg).unwrap();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", &xdg);
        }

        let eff = load_effective_config(&root).unwrap();
        assert_eq!(eff.cfg, GlobalConfig::default());
        assert!(matches!(
            eff.origin,
            ConfigOrigin::Default | ConfigOrigin::Global
        ));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn dotfile_in_parent_is_inherited_by_child_dir() {
        let root = unique_temp_dir("inherited");
        let parent = root.join("parent");
        let child = parent.join("child");
        fs::create_dir_all(&child).unwrap();

        let json = parent.join("bif.local.json");
        write(
            &json,
            r#"{"new_stamp_ids":["time"],"pretty":{"meta_keys":[]}}"#,
        );
        let dot = parent.join(".bif-config");
        write(&dot, "bif.local.json\n");

        let eff = load_effective_config(&child).unwrap();
        assert_eq!(eff.cfg.new_stamp_ids, vec!["time".to_string()]);
        match eff.origin {
            ConfigOrigin::Local {
                dotfile_path,
                json_path,
            } => {
                assert_eq!(dotfile_path, dot);
                assert_eq!(json_path, json.canonicalize().unwrap());
            }
            _ => panic!("expected local config"),
        }

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn invalid_json_returns_error() {
        let root = unique_temp_dir("invalid-json");
        fs::create_dir_all(&root).unwrap();

        let json = root.join("cfg.json");
        write(&json, "{ this is not json }");
        let dot = root.join(".bif-config");
        write(&dot, "cfg.json");

        let err = load_effective_config(&root).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid JSON"));
        assert!(msg.contains(&json.canonicalize().unwrap().display().to_string()));
        assert!(msg.contains(&dot.display().to_string()));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn missing_referenced_file_returns_error() {
        let root = unique_temp_dir("missing-json");
        fs::create_dir_all(&root).unwrap();

        let dot = root.join(".bif-config");
        write(&dot, "does-not-exist.json");

        let err = load_effective_config(&root).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("failed to resolve config JSON path"));
        assert!(msg.contains("does-not-exist.json"));
        assert!(msg.contains(&dot.display().to_string()));

        fs::remove_dir_all(&root).unwrap();
    }
}
