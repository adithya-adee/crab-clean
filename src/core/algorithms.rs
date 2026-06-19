//! Detection algorithms: duplicates (by content hash) and age-based selection.

use crate::core::model::{FileEntry, TimeBasis};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Find groups of byte-for-byte identical files.
///
/// Two-stage strategy: bucket by size first (cheap), then SHA-256 only the
/// buckets that have more than one file. Hashing runs in parallel via rayon.
/// Zero-length files are ignored. Each returned inner `Vec` is one set of
/// duplicates (length >= 2).
pub fn find_duplicates(entries: &[FileEntry]) -> Vec<Vec<FileEntry>> {
    // Stage 1: group by size.
    let mut by_size: HashMap<u64, Vec<&FileEntry>> = HashMap::new();
    for entry in entries {
        if entry.size == 0 {
            continue;
        }
        by_size.entry(entry.size).or_default().push(entry);
    }

    let candidates: Vec<Vec<&FileEntry>> = by_size
        .into_values()
        .filter(|group| group.len() > 1)
        .collect();

    // Stage 2: hash within each size bucket, in parallel.
    candidates
        .into_par_iter()
        .flat_map_iter(|group| {
            let mut by_hash: HashMap<String, Vec<FileEntry>> = HashMap::new();
            for entry in group {
                match hash_file(&entry.path) {
                    Ok(hash) => by_hash.entry(hash).or_default().push(entry.clone()),
                    Err(_) => { /* unreadable file: skip silently */ }
                }
            }
            by_hash
                .into_values()
                .filter(|g| g.len() > 1)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Select files whose `basis` timestamp is older than `age_days` days.
pub fn find_by_age(entries: &[FileEntry], age_days: u32, basis: TimeBasis) -> Vec<FileEntry> {
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(age_days as u64 * 86_400))
        .unwrap_or(UNIX_EPOCH);

    entries
        .iter()
        .filter(|entry| match entry.time(basis) {
            Some(t) => t < cutoff,
            None => false,
        })
        .cloned()
        .collect()
}

/// Streaming SHA-256 of a file's contents.
fn hash_file(path: &Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65_536]; // 64 KiB

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
