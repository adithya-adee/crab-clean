//! Safe deletion: move files to the OS trash (default) or delete permanently,
//! plus helpers for inspecting and emptying the system trash.

use crate::core::model::FileEntry;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// How a delete action removes files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeleteMode {
    /// Move to the OS recycle bin (recoverable). The safe default.
    #[default]
    Trash,
    /// Permanently remove with `std::fs::remove_file` (unrecoverable).
    Permanent,
}

impl DeleteMode {
    pub fn label(&self) -> &'static str {
        match self {
            DeleteMode::Trash => "Move to Trash (recoverable)",
            DeleteMode::Permanent => "Delete permanently (cannot be undone)",
        }
    }
}

/// Outcome of a delete operation.
#[derive(Debug, Default, Clone)]
pub struct DeleteReport {
    pub deleted: usize,
    pub bytes_freed: u64,
    /// Files that could not be removed, with the reason.
    pub failed: Vec<(PathBuf, String)>,
}

/// Delete the given entries using `mode`, returning a per-file report.
///
/// Operates file-by-file so a single failure never aborts the batch and every
/// failure is attributed to its path.
pub fn delete_files(entries: &[FileEntry], mode: DeleteMode) -> DeleteReport {
    let mut report = DeleteReport::default();

    for entry in entries {
        let result = match mode {
            DeleteMode::Trash => trash::delete(&entry.path).map_err(|e| e.to_string()),
            DeleteMode::Permanent => std::fs::remove_file(&entry.path).map_err(|e| e.to_string()),
        };

        match result {
            Ok(()) => {
                report.deleted += 1;
                report.bytes_freed += entry.size;
            }
            Err(reason) => report.failed.push((entry.path.clone(), reason)),
        }
    }

    report
}

/// Number of items currently in the system trash.
pub fn trash_item_count() -> Result<usize> {
    Ok(trash::os_limited::list()?.len())
}

/// Permanently purge everything in the system trash. Returns how many items
/// were purged.
pub fn empty_trash() -> Result<usize> {
    let items = trash::os_limited::list()?;
    let count = items.len();
    if count > 0 {
        trash::os_limited::purge_all(items)?;
    }
    Ok(count)
}
