use crate::cli::cli_error::CliError;
use crate::{cli, domain, storage};

fn require_tracked_log() -> Result<String, CliError> {
    match storage::tracked::get_tracked_file_path() {
        Ok(path) => Ok(path),
        Err(_err) => Err(CliError::InvalidArgs {
            message: "no tracked log. Run `bif init` or `bif track <name>`.".to_string(),
        }),
    }
}
#[derive(Debug, Clone, Copy)]
pub enum DeleteSpec {
    /// Delete the last N entries. `1` == last entry.
    CountFromEnd(usize),
    /// Delete the Nth entry from the end. `1` == last entry, `2` == second-to-last.
    IndexFromEnd(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum ReadSpec {
    /// Read the last N entries. `1` == last entry.
    CountFromEnd(usize),
    /// Read the Nth entry from the end. `1` == last entry, `2` == second-to-last.
    IndexFromEnd(usize),
}

pub enum Command {
    HELP,
    // Shows the help message
    INIT {
        name_of_new_log: Option<String>,
        config_path: Option<String>,
    },
    // Intialises a new .bif file.
    TRACK {
        name_of_log: String,
    },
    // Tracks an existing .bif file in the current working directory.
    NEW {
        body: String,
    },
    //Create a new entry.
    DELETE {
        spec: Option<DeleteSpec>,
    },
    // Deletes entries (default: last entry).
    READ {
        spec: Option<ReadSpec>,
        pretty: bool,
    },
    // Reads the current .bif file (default: entire file).
    CONFIG_SHOW,
    // Shows the active config origin (local/global/default) and paths.
    CONFIG_SET_LOCAL {
        path: String,
    },
    // Tracks a local config JSON path for this directory by writing `.bif-config`.
}

impl Command {
    /// Parses user input into a command.
    pub fn parse(input: &Vec<String>) -> Option<Command> {
        if input.is_empty() {
            return Some(Command::HELP);
        }

        match input[0].as_str() {
            "help" => Some(Command::HELP),
            "init" => {
                // bif init <optionalName>
                // - No name => default Record.bif
                // - One arg => use it
                // - More than one => invalid
                match input.len() {
                    1 => Some(Command::INIT {
                        name_of_new_log: None,
                        config_path: None,
                    }),
                    2 => Some(Command::INIT {
                        name_of_new_log: Some(input[1].clone()),
                        config_path: None,
                    }),
                    3 if input[1] == "--config" => Some(Command::INIT {
                        name_of_new_log: None,
                        config_path: Some(input[2].clone()),
                    }),
                    4 if input[2] == "--config" => Some(Command::INIT {
                        name_of_new_log: Some(input[1].clone()),
                        config_path: Some(input[3].clone()),
                    }),
                    _ => None,
                }
            }
            "track" => match input.len() {
                2 => Some(Command::TRACK {
                    name_of_log: input[1].clone(),
                }),
                _ => None,
            },
            "new" => {
                // `bif new <body...>`
                if input.len() < 2 {
                    return None;
                }
                Some(Command::NEW {
                    body: input[1..].join(" "),
                })
            }
            "delete" => {
                // Supported:
                // - `bif delete`      => delete last entry
                // - `bif delete 2`    => delete last 2 entries
                // - `bif delete -2`   => delete 2nd-to-last entry
                match input.len() {
                    1 => Some(Command::DELETE { spec: None }),
                    2 => {
                        let raw = input[1].trim();
                        if raw.is_empty() {
                            return None;
                        }

                        let n: i64 = raw.parse().ok()?;
                        if n == 0 {
                            return None;
                        }

                        if n > 0 {
                            Some(Command::DELETE {
                                spec: Some(DeleteSpec::CountFromEnd(n as usize)),
                            })
                        } else {
                            Some(Command::DELETE {
                                spec: Some(DeleteSpec::IndexFromEnd((-n) as usize)),
                            })
                        }
                    }
                    _ => None,
                }
            }
            "read" => {
                // Supported:
                // - `bif read`              => print entire file (raw records)
                // - `bif read 2`            => print last 2 entries (raw)
                // - `bif read -2`           => print 2nd-to-last entry (raw)
                // - `bif read --pretty`     => print entire file (pretty)
                // - `bif read --pretty 2`   => print last 2 entries (pretty)
                // - `bif read --pretty -2`  => print 2nd-to-last entry (pretty)

                let mut pretty = false;
                let mut args: Vec<&str> = Vec::new();

                for a in input.iter().skip(1) {
                    if a == "--pretty" {
                        pretty = true;
                    } else {
                        args.push(a);
                    }
                }

                match args.len() {
                    0 => Some(Command::READ { spec: None, pretty }),
                    1 => {
                        let raw = args[0].trim();
                        if raw.is_empty() {
                            return None;
                        }

                        let n: i64 = raw.parse().ok()?;
                        if n == 0 {
                            return None;
                        }

                        if n > 0 {
                            Some(Command::READ {
                                spec: Some(ReadSpec::CountFromEnd(n as usize)),
                                pretty,
                            })
                        } else {
                            Some(Command::READ {
                                spec: Some(ReadSpec::IndexFromEnd((-n) as usize)),
                                pretty,
                            })
                        }
                    }
                    _ => None,
                }
            }
            "config" => {
                // Supported:
                // - `bif config show`
                // - `bif config set <path> --local`
                //   where <path> is relative to the current directory
                match input.as_slice() {
                    [_, sub] if sub == "show" => Some(Command::CONFIG_SHOW),

                    // `bif config set ./mon_config.json --local`
                    [_, sub, path, flag] if sub == "set" && flag == "--local" => {
                        Some(Command::CONFIG_SET_LOCAL { path: path.clone() })
                    }
                    // Allow flag before path as well: `bif config set --local ./cfg.json`
                    [_, sub, flag, path] if sub == "set" && flag == "--local" => {
                        Some(Command::CONFIG_SET_LOCAL { path: path.clone() })
                    }

                    _ => None,
                }
            }
            _ => Some(Command::NEW {
                body: input.join(" "),
            }),
        }
    }

    /// Executes a command.
    pub fn execute(&self) -> Result<(), CliError> {
        let mut out = std::io::stdout();
        let mut err = std::io::stderr();
        self.execute_with_io(&mut out, &mut err)
    }

    /// Executes a command, writing output to the provided writers.
    ///
    /// This exists primarily to make CLI behavior testable without spawning a process.
    pub fn execute_with_io(
        &self,
        out: &mut dyn std::io::Write,
        err: &mut dyn std::io::Write,
    ) -> Result<(), CliError> {
        match self {
            Command::HELP => {
                cli::help::render();
                Ok(())
            }

            Command::INIT {
                name_of_new_log,
                config_path,
            } => {
                // Normalize/validate at the domain boundary.
                let file_name =
                    domain::log_filename::normalize_log_filename(name_of_new_log.as_deref())?;

                // Optionally create a local config JSON file (copied from global/default)
                // and track it via `.bif-config`.
                //
                // This must happen *before* creating the log file, so if we fail we don't leave
                // the repo half-initialized.
                if let Some(rel_str) = config_path.as_deref() {
                    let rel = std::path::Path::new(rel_str);
                    if rel.as_os_str().is_empty() {
                        return Err(CliError::InvalidArgs {
                            message: "config path cannot be empty".to_string(),
                        });
                    }
                    if rel.is_absolute() {
                        return Err(CliError::InvalidArgs {
                            message: "config path must be relative to the current directory"
                                .to_string(),
                        });
                    }

                    let cwd = std::env::current_dir()
                        .map_err(crate::storage::storage_error::StorageError::from)?;

                    // Load the GLOBAL config file if it exists, else default.
                    let src_cfg = crate::cli::config::GlobalConfig::load_global()
                        .map_err(crate::storage::storage_error::StorageError::from)?;
                    let json_bytes = src_cfg.canonical_json_bytes();

                    // Refuse to overwrite an existing file.
                    let dest_path = cwd.join(rel);
                    let mut f = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&dest_path)
                        .map_err(crate::storage::storage_error::StorageError::from)?;
                    use std::io::Write as _;
                    f.write_all(&json_bytes)
                        .map_err(crate::storage::storage_error::StorageError::from)?;

                    // Track local config path (overwrite `.bif-config`).
                    let dotfile = cwd.join(".bif-config");
                    std::fs::write(&dotfile, format!("{}\n", rel.display()))
                        .map_err(crate::storage::storage_error::StorageError::from)?;
                }

                // Storage operation: create the file in the current working directory.
                //
                // For now `fs_store` returns `std::io::Result`, so we map it into a
                // `StorageError::Io` via `From<std::io::Error> for StorageError`,
                // and then into `CliError::Storage` via `From<StorageError> for CliError`.
                //
                // Later, you can change `fs_store` to return `Result<_, StorageError>`
                // and delete this explicit mapping.
                let created_path = storage::fs_store::create_empty_record_file_in_cwd(&file_name)
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                // Persist tracked state in CWD.
                storage::tracked::set_tracked_file_path(created_path.to_string_lossy().as_ref())
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                println!("Initialized empty record: {}", created_path.display());
                Ok(())
            }

            Command::TRACK { name_of_log } => {
                // Reuse init filename normalization rules (adds `.bif` if missing;
                // rejects path separators).
                let file_name =
                    domain::log_filename::normalize_log_filename(Some(name_of_log.as_str()))?;

                let path = storage::tracked::resolve_existing_bif_in_cwd(&file_name)
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                storage::tracked::set_tracked_file_path(path.to_string_lossy().as_ref())
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                println!("Tracked record: {}", path.display());
                Ok(())
            }

            Command::NEW { body } => {
                let tracked = require_tracked_log()?;

                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|err| CliError::InvalidArgs {
                        message: format!("clock error: {err}"),
                    })?
                    .as_secs()
                    .to_string();

                let stamp =
                    domain::entry::Stamp::new(timestamp, domain::entry::EntryLevel::INFO, None);

                // Resolve effective config for the current directory (local config, inherited).
                // `_cfg_hash` must reflect the effective config bytes used for provider selection.
                let cwd = std::env::current_dir()
                    .map_err(crate::storage::storage_error::StorageError::from)?;
                let eff = crate::cli::config_resolver::load_effective_config(&cwd)
                    .map_err(crate::storage::storage_error::StorageError::from)?;
                let cfg_hash = eff.cfg.cfg_hash_hex();

                // Run configured stamp providers.
                let provider_ctx = crate::domain::stamp_provider::ProviderContext {
                    stamp: stamp.clone(),
                    cwd,
                };
                let reg = crate::domain::stamp_provider::Registry::default();
                let mut meta = reg.compute_meta_for_ids(&eff.cfg.new_stamp_ids, &provider_ctx);
                meta.insert("_cfg_hash".to_string(), cfg_hash);

                let mut entry = domain::entry::Entry::new(stamp, body.clone());
                entry.meta = meta;

                let record = entry.to_record();

                storage::fs_store::append_record_line(std::path::Path::new(&tracked), &record)
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                Ok(())
            }

            Command::DELETE { spec } => {
                let tracked = require_tracked_log()?;

                match spec {
                    None => {
                        storage::fs_store::delete_last_record_line(std::path::Path::new(&tracked))
                            .map_err(crate::storage::storage_error::StorageError::from)?;
                    }
                    Some(DeleteSpec::CountFromEnd(n)) => {
                        storage::fs_store::delete_last_n_record_lines(
                            std::path::Path::new(&tracked),
                            *n,
                        )
                        .map_err(crate::storage::storage_error::StorageError::from)?;
                    }
                    Some(DeleteSpec::IndexFromEnd(n)) => {
                        storage::fs_store::delete_record_line_by_index_from_end(
                            std::path::Path::new(&tracked),
                            *n,
                        )
                        .map_err(crate::storage::storage_error::StorageError::from)?;
                    }
                }

                Ok(())
            }

            Command::CONFIG_SHOW => {
                let cwd = std::env::current_dir()
                    .map_err(crate::storage::storage_error::StorageError::from)?;
                let eff = crate::cli::config_resolver::load_effective_config(&cwd)
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                match eff.origin {
                    crate::cli::config_resolver::ConfigOrigin::Local {
                        dotfile_path,
                        json_path,
                    } => {
                        writeln!(out, "local").map_err(|e| CliError::InvalidArgs {
                            message: format!("stdout write error: {e}"),
                        })?;
                        writeln!(out, ".bif-config: {}", dotfile_path.display()).map_err(|e| {
                            CliError::InvalidArgs {
                                message: format!("stdout write error: {e}"),
                            }
                        })?;
                        writeln!(out, "config: {}", json_path.display()).map_err(|e| {
                            CliError::InvalidArgs {
                                message: format!("stdout write error: {e}"),
                            }
                        })?;
                    }
                    crate::cli::config_resolver::ConfigOrigin::Global => {
                        writeln!(out, "global").map_err(|e| CliError::InvalidArgs {
                            message: format!("stdout write error: {e}"),
                        })?;
                        // Best-effort: show global path; if it doesn't exist, treat as default.
                        let p = crate::cli::config::default_config_path()
                            .map_err(crate::storage::storage_error::StorageError::from)?;
                        if p.exists() {
                            writeln!(out, "config: {}", p.display()).map_err(|e| {
                                CliError::InvalidArgs {
                                    message: format!("stdout write error: {e}"),
                                }
                            })?;
                        } else {
                            writeln!(out, "config: default").map_err(|e| {
                                CliError::InvalidArgs {
                                    message: format!("stdout write error: {e}"),
                                }
                            })?;
                        }
                    }
                    crate::cli::config_resolver::ConfigOrigin::Default => {
                        writeln!(out, "default").map_err(|e| CliError::InvalidArgs {
                            message: format!("stdout write error: {e}"),
                        })?;
                        writeln!(out, "config: default").map_err(|e| CliError::InvalidArgs {
                            message: format!("stdout write error: {e}"),
                        })?;
                    }
                }

                Ok(())
            }

            Command::CONFIG_SET_LOCAL { path } => {
                let cwd = std::env::current_dir()
                    .map_err(crate::storage::storage_error::StorageError::from)?;

                let rel = std::path::Path::new(path);
                if rel.as_os_str().is_empty() {
                    return Err(CliError::InvalidArgs {
                        message: "config path cannot be empty".to_string(),
                    });
                }
                if rel.is_absolute() {
                    return Err(CliError::InvalidArgs {
                        message: "config path must be relative to the current directory"
                            .to_string(),
                    });
                }

                let target = cwd.join(rel);
                let meta = std::fs::metadata(&target).map_err(|err| CliError::InvalidArgs {
                    message: format!(
                        "config path '{}' does not exist (relative to '{}'): {err}",
                        rel.display(),
                        cwd.display()
                    ),
                })?;
                if !meta.is_file() {
                    return Err(CliError::InvalidArgs {
                        message: format!("config path '{}' is not a file", rel.display()),
                    });
                }

                let dotfile = cwd.join(".bif-config");
                let existed = dotfile.exists();
                std::fs::write(&dotfile, format!("{}\n", rel.display()))
                    .map_err(|err| crate::storage::storage_error::StorageError::from(err))?;

                if existed {
                    writeln!(out, "Updated local config tracking: {}", rel.display()).map_err(
                        |e| CliError::InvalidArgs {
                            message: format!("stdout write error: {e}"),
                        },
                    )?;
                } else {
                    writeln!(out, "Tracked local config: {}", rel.display()).map_err(|e| {
                        CliError::InvalidArgs {
                            message: format!("stdout write error: {e}"),
                        }
                    })?;
                }

                Ok(())
            }

            Command::READ { spec, pretty } => {
                let tracked = require_tracked_log()?;

                // Default behavior MUST remain: raw record lines.
                if !pretty {
                    match spec {
                        None => {
                            let contents = std::fs::read_to_string(&tracked)
                                .map_err(crate::storage::storage_error::StorageError::from)?;
                            write!(out, "{contents}").map_err(|e| CliError::InvalidArgs {
                                message: format!("stdout write error: {e}"),
                            })?;
                        }
                        Some(ReadSpec::CountFromEnd(n)) => {
                            let lines = storage::fs_store::read_last_n_record_lines(
                                std::path::Path::new(&tracked),
                                *n,
                            )
                            .map_err(crate::storage::storage_error::StorageError::from)?;

                            if !lines.is_empty() {
                                writeln!(out, "{}", lines.join("\n")).map_err(|e| {
                                    CliError::InvalidArgs {
                                        message: format!("stdout write error: {e}"),
                                    }
                                })?;
                            }
                        }
                        Some(ReadSpec::IndexFromEnd(n)) => {
                            let line = storage::fs_store::read_record_line_by_index_from_end(
                                std::path::Path::new(&tracked),
                                *n,
                            )
                            .map_err(crate::storage::storage_error::StorageError::from)?;

                            writeln!(out, "{line}").map_err(|e| CliError::InvalidArgs {
                                message: format!("stdout write error: {e}"),
                            })?;
                        }
                    }

                    return Ok(());
                }

                // Pretty mode (presentation-layer only): parse record lines and render.
                // Prefer config-defined meta layout; fall back to legacy stamp format.
                let cwd = std::env::current_dir()
                    .map_err(crate::storage::storage_error::StorageError::from)?;
                let eff = crate::cli::config_resolver::load_effective_config(&cwd)
                    .map_err(crate::storage::storage_error::StorageError::from)?;
                let current_cfg_hash = eff.cfg.cfg_hash_hex();
                let pretty_cfg = eff.cfg.pretty.clone();

                fn render_pretty_stamp_from_meta(
                    entry: &domain::entry::Entry,
                    pretty_cfg: &crate::cli::config::PrettyConfig,
                ) -> String {
                    let mut parts: Vec<String> = Vec::new();
                    for k in &pretty_cfg.meta_keys {
                        let v = entry.meta.get(k).cloned().unwrap_or_default();
                        parts.push(v);
                    }
                    parts.join(&pretty_cfg.meta_sep)
                }

                match spec {
                    None => {
                        let contents = std::fs::read_to_string(&tracked)
                            .map_err(crate::storage::storage_error::StorageError::from)?;

                        for line in contents.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }

                            let entry = domain::entry::Entry::from_record(line).map_err(|err| {
                                CliError::InvalidArgs {
                                    message: format!("invalid record line: {err:?} | line: {line}"),
                                }
                            })?;

                            if !entry.meta.is_empty() {
                                if let Some(stored) = entry.meta.get("_cfg_hash") {
                                    if stored != &current_cfg_hash {
                                        writeln!(
                                            err,
                                            "bif: pretty: cfg hash mismatch (entry _cfg_hash={stored}, current={current_cfg_hash})"
                                        )
                                        .ok();
                                    }
                                }
                            }

                            let stamp = if !pretty_cfg.meta_keys.is_empty() {
                                render_pretty_stamp_from_meta(&entry, &pretty_cfg)
                            } else {
                                domain::stamp_format::render_stamp(
                                    &entry.stamp,
                                    &pretty_cfg.legacy_stamp_format,
                                )
                            };
                            writeln!(out, "{stamp}\t{}", entry.body).map_err(|e| {
                                CliError::InvalidArgs {
                                    message: format!("stdout write error: {e}"),
                                }
                            })?;
                        }
                    }
                    Some(ReadSpec::CountFromEnd(n)) => {
                        let lines = storage::fs_store::read_last_n_record_lines(
                            std::path::Path::new(&tracked),
                            *n,
                        )
                        .map_err(crate::storage::storage_error::StorageError::from)?;

                        for line in lines {
                            let entry =
                                domain::entry::Entry::from_record(&line).map_err(|err| {
                                    CliError::InvalidArgs {
                                        message: format!(
                                            "invalid record line: {err:?} | line: {line}"
                                        ),
                                    }
                                })?;

                            if !entry.meta.is_empty() {
                                if let Some(stored) = entry.meta.get("_cfg_hash") {
                                    if stored != &current_cfg_hash {
                                        writeln!(
                                            err,
                                            "bif: pretty: cfg hash mismatch (entry _cfg_hash={stored}, current={current_cfg_hash})"
                                        )
                                        .ok();
                                    }
                                }
                            }

                            let stamp = if !pretty_cfg.meta_keys.is_empty() {
                                render_pretty_stamp_from_meta(&entry, &pretty_cfg)
                            } else {
                                domain::stamp_format::render_stamp(
                                    &entry.stamp,
                                    &pretty_cfg.legacy_stamp_format,
                                )
                            };
                            writeln!(out, "{stamp}\t{}", entry.body).map_err(|e| {
                                CliError::InvalidArgs {
                                    message: format!("stdout write error: {e}"),
                                }
                            })?;
                        }
                    }
                    Some(ReadSpec::IndexFromEnd(n)) => {
                        let line = storage::fs_store::read_record_line_by_index_from_end(
                            std::path::Path::new(&tracked),
                            *n,
                        )
                        .map_err(crate::storage::storage_error::StorageError::from)?;

                        let entry = domain::entry::Entry::from_record(&line).map_err(|err| {
                            CliError::InvalidArgs {
                                message: format!("invalid record line: {err:?} | line: {line}"),
                            }
                        })?;

                        if !entry.meta.is_empty() {
                            if let Some(stored) = entry.meta.get("_cfg_hash") {
                                if stored != &current_cfg_hash {
                                    writeln!(
                                        err,
                                        "bif: pretty: cfg hash mismatch (entry _cfg_hash={stored}, current={current_cfg_hash})"
                                    )
                                    .ok();
                                }
                            }
                        }

                        let stamp = if !pretty_cfg.meta_keys.is_empty() {
                            render_pretty_stamp_from_meta(&entry, &pretty_cfg)
                        } else {
                            domain::stamp_format::render_stamp(
                                &entry.stamp,
                                &pretty_cfg.legacy_stamp_format,
                            )
                        };
                        writeln!(out, "{stamp}\t{}", entry.body).map_err(|e| {
                            CliError::InvalidArgs {
                                message: format!("stdout write error: {e}"),
                            }
                        })?;
                    }
                }

                Ok(())
            }
        }
    }
}
