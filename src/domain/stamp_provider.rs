use std::collections::BTreeMap;

use crate::domain::entry::{EntryLevel, Stamp};

/// Computes additional metadata ("stamps") to store into `Entry.meta` at `bif new` time.
///
/// Providers are identified by a stable string ID (e.g. `"time"`, `"cwd"`).
///
/// IMPORTANT: Provider outputs must be one-line safe. This is enforced at record-encoding
/// time via `Entry::to_record` field escaping, but providers should still avoid returning
/// giant blobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderContext {
    pub stamp: Stamp,
    pub cwd: std::path::PathBuf,
}

pub trait StampProvider: Send + Sync {
    /// Stable identifier used in config and persisted meta keys.
    fn id(&self) -> &'static str;

    /// Compute the stamp value.
    fn compute(&self, ctx: &ProviderContext) -> Result<String, String>;
}

pub struct Registry {
    providers: Vec<Box<dyn StampProvider>>,
}

impl Registry {
    pub fn default() -> Self {
        Self {
            providers: vec![
                Box::new(TimeProvider),
                Box::new(DateProvider),
                Box::new(DateTimeProvider),
                Box::new(LevelProvider),
                Box::new(SourceProvider),
                Box::new(CwdProvider),
            ],
        }
    }

    pub fn get(&self, id: &str) -> Option<&dyn StampProvider> {
        self.providers
            .iter()
            .map(|p| p.as_ref())
            .find(|p| p.id() == id)
    }

    /// Executes providers in the given order and returns a meta map of `id -> value`.
    ///
    /// Unknown provider IDs are ignored.
    /// Provider failures are ignored (fail-closed), to keep `bif new` reliable.
    pub fn compute_meta_for_ids(
        &self,
        ids: &[String],
        ctx: &ProviderContext,
    ) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();

        for id in ids {
            let Some(p) = self.get(id.as_str()) else {
                continue;
            };

            if let Ok(v) = p.compute(ctx) {
                out.insert(p.id().to_string(), v);
            }
        }

        out
    }
}

struct TimeProvider;
impl StampProvider for TimeProvider {
    fn id(&self) -> &'static str {
        "time"
    }

    fn compute(&self, ctx: &ProviderContext) -> Result<String, String> {
        Ok(ctx.stamp.timestamp.clone())
    }
}

struct LevelProvider;
impl StampProvider for LevelProvider {
    fn id(&self) -> &'static str {
        "level"
    }

    fn compute(&self, ctx: &ProviderContext) -> Result<String, String> {
        Ok(match ctx.stamp.level {
            EntryLevel::DEBUG => "DEBUG",
            EntryLevel::INFO => "INFO",
            EntryLevel::WARN => "WARN",
            EntryLevel::ERROR => "ERROR",
        }
        .to_string())
    }
}

struct SourceProvider;
impl StampProvider for SourceProvider {
    fn id(&self) -> &'static str {
        "source"
    }

    fn compute(&self, ctx: &ProviderContext) -> Result<String, String> {
        Ok(ctx.stamp.source.clone().unwrap_or_default())
    }
}

struct CwdProvider;
impl StampProvider for CwdProvider {
    fn id(&self) -> &'static str {
        "cwd"
    }

    fn compute(&self, ctx: &ProviderContext) -> Result<String, String> {
        Ok(ctx.cwd.to_string_lossy().to_string())
    }
}

struct DateProvider;
impl StampProvider for DateProvider {
    fn id(&self) -> &'static str {
        "date"
    }

    fn compute(&self, ctx: &ProviderContext) -> Result<String, String> {
        // Use local time; matches typical CLI expectations.
        use chrono::{Local, TimeZone};
        let ts = ctx
            .stamp
            .timestamp
            .parse::<i64>()
            .map_err(|e| format!("invalid timestamp: {e}"))?;
        let dt = Local
            .timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| "timestamp out of range".to_string())?;
        Ok(dt.format("%Y-%m-%d").to_string())
    }
}

struct DateTimeProvider;
impl StampProvider for DateTimeProvider {
    fn id(&self) -> &'static str {
        "datetime"
    }

    fn compute(&self, ctx: &ProviderContext) -> Result<String, String> {
        use chrono::{Local, TimeZone};
        let ts = ctx
            .stamp
            .timestamp
            .parse::<i64>()
            .map_err(|e| format!("invalid timestamp: {e}"))?;
        let dt = Local
            .timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| "timestamp out of range".to_string())?;
        // ISO-ish local time.
        Ok(dt.format("%Y-%m-%dT%H:%M:%S%:z").to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> ProviderContext {
        ProviderContext {
            stamp: Stamp::new("0".to_string(), EntryLevel::INFO, Some("src".to_string())),
            cwd: std::path::PathBuf::from("/tmp"),
        }
    }

    #[test]
    fn registry_computes_known_providers_in_order() {
        let reg = Registry::default();
        let ids = vec!["level".to_string(), "time".to_string(), "nope".to_string()];
        let m = reg.compute_meta_for_ids(&ids, &ctx());

        assert_eq!(m.get("level"), Some(&"INFO".to_string()));
        assert_eq!(m.get("time"), Some(&"0".to_string()));
        assert!(!m.contains_key("nope"));
    }

    #[test]
    fn date_and_datetime_are_parseable() {
        let reg = Registry::default();
        let c = ctx();

        let date = reg.get("date").unwrap().compute(&c).unwrap();
        // Local time zone may render epoch as 1969-12-31 in negative offsets.
        assert_eq!(date.len(), 10);
        assert!(date.chars().all(|ch| ch.is_ascii_digit() || ch == '-'));

        let dt = reg.get("datetime").unwrap().compute(&c).unwrap();
        assert!(dt.contains('T'));
    }
}
