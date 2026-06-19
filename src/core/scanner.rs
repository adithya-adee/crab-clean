//! Filesystem traversal that produces [`FileEntry`] records.

use crate::core::model::FileEntry;
use crate::error::{CrabError, Result};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// Options controlling how the directory tree is walked.
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    /// Maximum recursion depth. `None` means unlimited.
    pub max_depth: Option<usize>,
    /// Include dot-files and dot-directories.
    pub include_hidden: bool,
    /// Follow symbolic links (off by default to avoid cycles / double counting).
    pub follow_symlinks: bool,
}

fn is_hidden(entry: &DirEntry) -> bool {
    // Never treat the root (depth 0) as hidden, otherwise scanning "." prunes
    // everything immediately.
    entry.depth() > 0
        && entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
}

/// Walk `root` and collect every regular file as a [`FileEntry`].
///
/// `on_progress` is invoked periodically with the running file count so a UI can
/// show live progress. Unreadable entries are skipped rather than aborting the
/// whole scan (this is the bug the old `.unwrap()`-based scanner had).
pub fn scan<F>(root: &Path, opts: &ScanOptions, mut on_progress: F) -> Result<Vec<FileEntry>>
where
    F: FnMut(usize),
{
    if !root.is_dir() {
        return Err(CrabError::InvalidArgument(format!(
            "'{}' is not a directory or does not exist",
            root.display()
        )));
    }

    let mut walker = WalkDir::new(root).follow_links(opts.follow_symlinks);
    if let Some(depth) = opts.max_depth {
        walker = walker.max_depth(depth);
    }

    let include_hidden = opts.include_hidden;
    let mut entries = Vec::new();
    let mut count = 0usize;

    for dir_entry in walker
        .into_iter()
        .filter_entry(|e| include_hidden || !is_hidden(e))
    {
        let dir_entry = match dir_entry {
            Ok(e) => e,
            Err(_) => continue, // permission denied, broken link, etc.
        };

        if !dir_entry.file_type().is_file() {
            continue;
        }

        let metadata = match dir_entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let path = dir_entry.into_path();
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        entries.push(FileEntry {
            size: metadata.len(),
            modified: metadata.modified().ok(),
            accessed: metadata.accessed().ok(),
            created: metadata.created().ok(),
            extension,
            path,
        });

        count += 1;
        if count.is_multiple_of(256) {
            on_progress(count);
        }
    }

    on_progress(count);
    Ok(entries)
}
