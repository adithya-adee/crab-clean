//! Interactive result filtering (extension, size, date range, name).

use crate::core::model::{FileEntry, TimeBasis};
use std::time::SystemTime;

/// A composable predicate applied to scan results in the review screen.
///
/// Every field is optional; an empty filter matches everything.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// Lower-cased extensions (without dot). Empty means "any".
    pub extensions: Vec<String>,
    /// Case-insensitive substring the file name must contain.
    pub name_contains: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    /// Which timestamp the date bounds apply to.
    pub time_basis: TimeBasis,
    pub after: Option<SystemTime>,
    pub before: Option<SystemTime>,
}

impl Filter {
    /// Does this entry pass all active criteria?
    pub fn matches(&self, entry: &FileEntry) -> bool {
        if !self.extensions.is_empty() {
            let ext = entry.ext_str();
            if !self.extensions.iter().any(|e| e == ext) {
                return false;
            }
        }

        if let Some(needle) = &self.name_contains
            && !entry.file_name().to_lowercase().contains(needle)
        {
            return false;
        }

        if let Some(min) = self.min_size
            && entry.size < min
        {
            return false;
        }
        if let Some(max) = self.max_size
            && entry.size > max
        {
            return false;
        }

        if self.after.is_some() || self.before.is_some() {
            match entry.time(self.time_basis) {
                Some(t) => {
                    if let Some(after) = self.after
                        && t < after
                    {
                        return false;
                    }
                    if let Some(before) = self.before
                        && t > before
                    {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }

    /// Is any criterion set?
    pub fn is_active(&self) -> bool {
        !self.extensions.is_empty()
            || self.name_contains.is_some()
            || self.min_size.is_some()
            || self.max_size.is_some()
            || self.after.is_some()
            || self.before.is_some()
    }
}

/// Parse a human size string like `10`, `512KB`, `1.5 GB` into bytes.
pub fn parse_size(input: &str) -> Option<u64> {
    let s = input.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }
    let split = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
    let (num, unit) = s.split_at(split);
    let value: f64 = num.trim().parse().ok()?;
    let multiplier: f64 = match unit.trim() {
        "" | "b" => 1.0,
        "k" | "kb" => 1_000.0,
        "kib" => 1_024.0,
        "m" | "mb" => 1_000_000.0,
        "mib" => 1_048_576.0,
        "g" | "gb" => 1_000_000_000.0,
        "gib" => 1_073_741_824.0,
        "t" | "tb" => 1_000_000_000_000.0,
        _ => return None,
    };
    Some((value * multiplier) as u64)
}

/// Parse a `YYYY-MM-DD` date (interpreted in local time, start of day).
pub fn parse_date(input: &str) -> Option<SystemTime> {
    use chrono::{Local, NaiveDate, TimeZone};
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    let date = NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?;
    let naive = date.and_hms_opt(0, 0, 0)?;
    let local = Local.from_local_datetime(&naive).single()?;
    Some(local.into())
}
