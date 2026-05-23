use crate::domain::stamp_format::StampFormat;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Global, user-level configuration for bif.
///
/// Loaded from a config file (JSON) located in a stable, user-specific directory.
///
/// NOTE: This module is intentionally *not* wired into commands yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Order of stamp provider IDs to compute at `new` time.
    #[serde(default)]
    pub new_stamp_ids: Vec<String>,

    /// Pretty rendering layout.
    #[serde(default)]
    pub pretty: PrettyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrettyConfig {
    /// Which meta keys to display in `read --pretty`, and in what order.
    ///
    /// If empty, falls back to the legacy `pretty_stamp_format` rendering over the canonical
    /// `Stamp` fields.
    #[serde(default)]
    pub meta_keys: Vec<String>,

    /// Separator between meta fields.
    #[serde(default = "default_meta_sep")]
    pub meta_sep: String,

    /// Fallback presentation-layer stamp format for legacy entries (no meta) or when `meta_keys`
    /// is empty.
    #[serde(default = "StampFormat::default_pretty")]
    pub legacy_stamp_format: StampFormat,
}

fn default_meta_sep() -> String {
    " ".to_string()
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            new_stamp_ids: Vec::new(),
            pretty: PrettyConfig::default(),
        }
    }
}

impl Default for PrettyConfig {
    fn default() -> Self {
        Self {
            meta_keys: Vec::new(),
            meta_sep: default_meta_sep(),
            legacy_stamp_format: StampFormat::default_pretty(),
        }
    }
}

impl GlobalConfig {
    /// Reads the global config from the default location.
    ///
    /// If no config file exists, returns the default config.
    pub fn load_global() -> std::io::Result<Self> {
        let path = default_config_path()?;
        match std::fs::read_to_string(&path) {
            Ok(s) => {
                let cfg: Self = serde_json::from_str(&s).map_err(|err| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
                })?;
                Ok(cfg)
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err),
        }
    }

    /// Returns a canonical, stable JSON encoding of this config.
    ///
    /// This is used as the input bytes to compute `_cfg_hash`.
    pub fn canonical_json_bytes(&self) -> Vec<u8> {
        // We want stable bytes across runs:
        // - use a stable serialization (serde_json)
        // - ensure that any maps are deterministic (we only have Vec and enums today)
        // - avoid pretty-printing differences
        serde_json::to_vec(self).expect("GlobalConfig must be JSON-serializable")
    }

    /// Computes SHA-256 over the canonical JSON bytes, returned as lowercase hex.
    pub fn cfg_hash_hex(&self) -> String {
        sha256_hex(&self.canonical_json_bytes())
    }
}

/// Returns the default config path.
///
/// Strategy:
/// - Prefer XDG_CONFIG_HOME on Unix.
/// - Otherwise use HOME/.config.
/// - On Windows, use APPDATA when present.
///
/// File name is `bif/config.json`.
pub fn default_config_path() -> std::io::Result<PathBuf> {
    // Windows: %APPDATA%\bif\config.json
    if cfg!(windows) {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Ok(PathBuf::from(appdata).join("bif").join("config.json"));
        }
    }

    // XDG: $XDG_CONFIG_HOME/bif/config.json
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.trim().is_empty() {
            return Ok(PathBuf::from(xdg).join("bif").join("config.json"));
        }
    }

    // Fallback: $HOME/.config/bif/config.json
    let home = std::env::var("HOME").map_err(|_err| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME not set; cannot resolve default config path",
        )
    })?;

    Ok(PathBuf::from(home)
        .join(".config")
        .join("bif")
        .join("config.json"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();

    // Lowercase hex.
    let mut s = String::with_capacity(out.len() * 2);
    for b in out {
        use std::fmt::Write as _;
        write!(&mut s, "{:02x}", b).expect("writing to string must not fail");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_config_same_hash() {
        let a = GlobalConfig::default();
        let b = GlobalConfig::default();
        assert_eq!(a.canonical_json_bytes(), b.canonical_json_bytes());
        assert_eq!(a.cfg_hash_hex(), b.cfg_hash_hex());
    }

    #[test]
    fn hash_changes_if_config_changes() {
        let a = GlobalConfig::default();
        let mut b = GlobalConfig::default();
        b.new_stamp_ids.push("time".to_string());

        assert_ne!(a.canonical_json_bytes(), b.canonical_json_bytes());
        assert_ne!(a.cfg_hash_hex(), b.cfg_hash_hex());
    }

    #[test]
    fn hash_is_lowercase_hex_len_64() {
        let h = GlobalConfig::default().cfg_hash_hex();
        assert_eq!(h.len(), 64);
        assert!(
            h.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }
}
