use crate::domain::entry::{EntryLevel, Stamp};

/// Presentation-layer stamp formatting.
///
/// IMPORTANT: This does NOT change the on-disk stamp record format.
/// It is only used when rendering stamps for human-facing output.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StampFormat {
    pub parts: Vec<StampPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StampPart {
    Literal(String),

    // Time parts (UTC) derived from `Stamp.timestamp` epoch seconds when possible.
    TimeHH,
    TimeMM,
    TimeSS,

    // Date parts (UTC) derived from `Stamp.timestamp` epoch seconds when possible.
    DateDD,
    DateMM,
    DateYYYY,

    // Domain fields.
    Level,
    Source,
}

impl StampFormat {
    pub fn new(parts: Vec<StampPart>) -> Self {
        Self { parts }
    }

    /// Default "pretty" format used by CLI presentation modes.
    ///
    /// Chosen to be stable, compact, and to include level.
    pub fn default_pretty() -> Self {
        Self {
            parts: vec![
                StampPart::Literal("[".to_string()),
                StampPart::DateYYYY,
                StampPart::Literal("-".to_string()),
                StampPart::DateMM,
                StampPart::Literal("-".to_string()),
                StampPart::DateDD,
                StampPart::Literal(" ".to_string()),
                StampPart::TimeHH,
                StampPart::Literal(":".to_string()),
                StampPart::TimeMM,
                StampPart::Literal(":".to_string()),
                StampPart::TimeSS,
                StampPart::Literal("] ".to_string()),
                StampPart::Level,
            ],
        }
    }
}

pub fn render_stamp(stamp: &Stamp, fmt: &StampFormat) -> String {
    let mut out = String::new();

    let epoch_secs = stamp.timestamp.trim().parse::<i64>().ok();
    let dt = epoch_secs.and_then(epoch_seconds_to_utc_components);

    for part in &fmt.parts {
        match part {
            StampPart::Literal(s) => out.push_str(s),

            StampPart::TimeHH => push_time(&mut out, dt.as_ref().map(|d| d.hh), &stamp.timestamp),
            StampPart::TimeMM => push_time(&mut out, dt.as_ref().map(|d| d.mm), &stamp.timestamp),
            StampPart::TimeSS => push_time(&mut out, dt.as_ref().map(|d| d.ss), &stamp.timestamp),

            StampPart::DateDD => push_date(&mut out, dt.as_ref().map(|d| d.dd), &stamp.timestamp),
            StampPart::DateMM => push_date(&mut out, dt.as_ref().map(|d| d.mo), &stamp.timestamp),
            StampPart::DateYYYY => match dt.as_ref() {
                Some(d) => out.push_str(&format!("{:04}", d.yyyy)),
                None => out.push_str(&stamp.timestamp),
            },

            StampPart::Level => out.push_str(entry_level_to_str(&stamp.level)),
            StampPart::Source => {
                if let Some(src) = &stamp.source {
                    out.push_str(src);
                }
            }
        }
    }

    out
}

fn entry_level_to_str(level: &EntryLevel) -> &'static str {
    match level {
        EntryLevel::DEBUG => "DEBUG",
        EntryLevel::INFO => "INFO",
        EntryLevel::WARN => "WARN",
        EntryLevel::ERROR => "ERROR",
    }
}

#[derive(Debug, Clone, Copy)]
struct UtcParts {
    yyyy: i32,
    mo: u32,
    dd: u32,
    hh: u32,
    mm: u32,
    ss: u32,
}

/// Converts epoch seconds to basic UTC calendar/time components.
///
/// - No external deps.
/// - Limited but sufficient for CLI display.
/// - Returns `None` on negative values (pre-epoch) or overflow.
fn epoch_seconds_to_utc_components(secs: i64) -> Option<UtcParts> {
    if secs < 0 {
        return None;
    }

    let secs_u: u64 = secs.try_into().ok()?;

    let days = secs_u / 86_400;
    let rem = secs_u % 86_400;

    let hh = (rem / 3_600) as u32;
    let mm = ((rem % 3_600) / 60) as u32;
    let ss = (rem % 60) as u32;

    let (yyyy, mo, dd) = civil_from_days(days)?;

    Some(UtcParts {
        yyyy,
        mo,
        dd,
        hh,
        mm,
        ss,
    })
}

/// Howard Hinnant's algorithm for converting days since Unix epoch to civil date.
///
/// Returns (year, month, day), with month in 1..=12.
fn civil_from_days(days_since_unix_epoch: u64) -> Option<(i32, u32, u32)> {
    // Convert to days since 1970-01-01 civil.
    // We shift to days since 0000-03-01 to simplify leap year logic.
    // This implementation is adapted for unsigned input.

    // 1970-01-01 is 719468 days after 0000-03-01 in this formulation.
    let z: i64 = (days_since_unix_epoch as i64) + 719468;

    // These computations are safe in i64 for realistic ranges.
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = mp + if mp < 10 { 3 } else { -9 }; // [1, 12]
    let year = (y + if m <= 2 { 1 } else { 0 }) as i32;

    let month: u32 = (m as i64).try_into().ok()?;
    let day: u32 = (d as i64).try_into().ok()?;

    Some((year, month, day))
}

fn push_time(out: &mut String, v: Option<u32>, raw_ts: &str) {
    match v {
        Some(x) => out.push_str(&format!("{:02}", x)),
        None => out.push_str(raw_ts),
    }
}

fn push_date(out: &mut String, v: Option<u32>, raw_ts: &str) {
    match v {
        Some(x) => out.push_str(&format!("{:02}", x)),
        None => out.push_str(raw_ts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_default_pretty_epoch_zero_is_1970_01_01_midnight_utc() {
        let stamp = Stamp::new("0".to_string(), EntryLevel::INFO, None);
        let s = render_stamp(&stamp, &StampFormat::default_pretty());
        assert_eq!(s, "[1970-01-01 00:00:00] INFO");
    }

    #[test]
    fn render_includes_source_when_requested() {
        let stamp = Stamp::new("0".to_string(), EntryLevel::WARN, Some("cli".to_string()));
        let fmt = StampFormat::new(vec![
            StampPart::Level,
            StampPart::Literal("(".to_string()),
            StampPart::Source,
            StampPart::Literal(")".to_string()),
        ]);

        let s = render_stamp(&stamp, &fmt);
        assert_eq!(s, "WARN(cli)");
    }

    #[test]
    fn render_time_part_falls_back_to_raw_timestamp_if_unparseable() {
        let stamp = Stamp::new("not-a-number".to_string(), EntryLevel::DEBUG, None);
        let fmt = StampFormat::new(vec![StampPart::TimeHH]);
        let s = render_stamp(&stamp, &fmt);
        assert_eq!(s, "not-a-number");
    }
}
