//! Core data types shared across scanning, filtering and the TUI.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// A single scanned file together with the metadata the tool needs.
///
/// Metadata is read exactly once (during the scan) and cached here so the rest
/// of the program never has to touch the filesystem again until deletion.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub created: Option<SystemTime>,
    /// Lower-cased extension without the leading dot (`None` when absent).
    pub extension: Option<String>,
}

impl FileEntry {
    /// The file name as a lossy string (never panics on non-UTF8).
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Extension as a displayable string (`""` when there is none).
    pub fn ext_str(&self) -> &str {
        self.extension.as_deref().unwrap_or("")
    }

    /// Returns the timestamp for the given basis, if available.
    pub fn time(&self, basis: TimeBasis) -> Option<SystemTime> {
        match basis {
            TimeBasis::Modified => self.modified,
            TimeBasis::Accessed => self.accessed,
            TimeBasis::Created => self.created,
        }
    }
}

/// What kind of cleanup scan to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMode {
    /// Exact duplicates detected by size bucket + SHA-256.
    Duplicates,
    /// Files not *accessed* for N days.
    Unused,
    /// Files not *modified* for N days.
    Old,
    /// Every file, to be narrowed down with filters.
    AllFiles,
}

impl ScanMode {
    pub const ALL: [ScanMode; 4] = [
        ScanMode::Duplicates,
        ScanMode::Unused,
        ScanMode::Old,
        ScanMode::AllFiles,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            ScanMode::Duplicates => "Duplicate files",
            ScanMode::Unused => "Unused files",
            ScanMode::Old => "Old files",
            ScanMode::AllFiles => "All files (browse + filter)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ScanMode::Duplicates => "Identical content found via SHA-256 hashing.",
            ScanMode::Unused => "Not accessed for the given number of days.",
            ScanMode::Old => "Not modified for the given number of days.",
            ScanMode::AllFiles => "List everything, then narrow with filters.",
        }
    }

    /// Whether this mode uses the age (days) input on the home screen.
    pub fn uses_age(&self) -> bool {
        matches!(self, ScanMode::Unused | ScanMode::Old)
    }
}

/// Which timestamp a comparison or filter should use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeBasis {
    #[default]
    Modified,
    Accessed,
    Created,
}
