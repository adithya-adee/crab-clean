//! Integration tests for crab-clean's core logic.
//!
//! SAFETY: every test operates exclusively on throwaway files inside a
//! `tempfile::tempdir()` — never on real user files. The destructive delete
//! test uses `DeleteMode::Permanent` on these mock files only.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crab_clean::core::algorithms::{find_by_age, find_duplicates};
use crab_clean::core::delete::{DeleteMode, delete_files};
use crab_clean::core::filter::{Filter, parse_date, parse_size};
use crab_clean::core::model::{FileEntry, TimeBasis};
use crab_clean::core::scanner::{ScanOptions, scan};
use tempfile::tempdir;

fn write_file(dir: &Path, name: &str, contents: &[u8]) -> PathBuf {
    let path = dir.join(name);
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(contents).unwrap();
    path
}

fn entry(path: PathBuf, size: u64, ext: Option<&str>) -> FileEntry {
    FileEntry {
        path,
        size,
        modified: None,
        accessed: None,
        created: None,
        extension: ext.map(|s| s.to_string()),
    }
}

#[test]
fn scan_collects_regular_files_and_skips_hidden() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "a.txt", b"hello");
    write_file(dir.path(), "b.txt", b"world!!");
    write_file(dir.path(), ".secret", b"nope");

    let files = scan(dir.path(), &ScanOptions::default(), |_| {}).unwrap();

    // Hidden file is excluded by default.
    assert_eq!(files.len(), 2);
    assert!(files.iter().all(|f| !f.file_name().starts_with('.')));
    assert!(files.iter().any(|f| f.size == 5)); // "hello"
}

#[test]
fn scan_can_include_hidden() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "visible.txt", b"x");
    write_file(dir.path(), ".hidden", b"y");

    let opts = ScanOptions {
        include_hidden: true,
        ..ScanOptions::default()
    };
    let files = scan(dir.path(), &opts, |_| {}).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn scan_rejects_non_directory() {
    let dir = tempdir().unwrap();
    let file = write_file(dir.path(), "f.txt", b"x");
    assert!(scan(&file, &ScanOptions::default(), |_| {}).is_err());
}

#[test]
fn find_duplicates_groups_identical_content() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "1.txt", b"same content here");
    write_file(dir.path(), "2.txt", b"same content here");
    write_file(dir.path(), "3.txt", b"same content here");
    write_file(dir.path(), "unique.txt", b"totally different");

    let files = scan(dir.path(), &ScanOptions::default(), |_| {}).unwrap();
    let groups = find_duplicates(&files);

    assert_eq!(groups.len(), 1, "expected exactly one duplicate group");
    assert_eq!(groups[0].len(), 3, "expected three identical files");
}

#[test]
fn find_duplicates_ignores_same_size_different_content() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "a.txt", b"AAAA"); // same size...
    write_file(dir.path(), "b.txt", b"BBBB"); // ...different bytes
    let files = scan(dir.path(), &ScanOptions::default(), |_| {}).unwrap();
    assert!(find_duplicates(&files).is_empty());
}

#[test]
fn find_by_age_excludes_fresh_files() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "new.txt", b"fresh");
    let files = scan(dir.path(), &ScanOptions::default(), |_| {}).unwrap();

    // Nothing just-created is 100 years old.
    let old = find_by_age(&files, 36_500, TimeBasis::Modified);
    assert!(old.is_empty());
}

#[test]
fn parse_size_understands_units() {
    assert_eq!(parse_size("10"), Some(10));
    assert_eq!(parse_size("1kb"), Some(1_000));
    assert_eq!(parse_size("1kib"), Some(1_024));
    assert_eq!(parse_size("2.5mb"), Some(2_500_000));
    assert_eq!(parse_size("  3 GB "), Some(3_000_000_000));
    assert_eq!(parse_size("nonsense"), None);
    assert_eq!(parse_size(""), None);
}

#[test]
fn parse_date_accepts_iso_dates() {
    assert!(parse_date("2020-01-01").is_some());
    assert!(parse_date("not-a-date").is_none());
    assert!(parse_date("").is_none());
}

#[test]
fn filter_matches_extension_and_size() {
    let jpg = entry(PathBuf::from("photo.jpg"), 5_000_000, Some("jpg"));
    let txt = entry(PathBuf::from("notes.txt"), 100, Some("txt"));

    let only_jpg = Filter {
        extensions: vec!["jpg".to_string()],
        ..Filter::default()
    };
    assert!(only_jpg.matches(&jpg));
    assert!(!only_jpg.matches(&txt));

    let big = Filter {
        min_size: Some(1_000_000),
        ..Filter::default()
    };
    assert!(big.matches(&jpg));
    assert!(!big.matches(&txt));

    assert!(Filter::default().matches(&txt)); // empty filter matches all
}

#[test]
fn delete_permanent_removes_mock_files_only() {
    // Operates strictly on mock files inside a tempdir.
    let dir = tempdir().unwrap();
    let p1 = write_file(dir.path(), "trash1.txt", b"junk");
    let p2 = write_file(dir.path(), "trash2.txt", b"junk2");

    let files = scan(dir.path(), &ScanOptions::default(), |_| {}).unwrap();
    assert_eq!(files.len(), 2);

    let report = delete_files(&files, DeleteMode::Permanent);
    assert_eq!(report.deleted, 2);
    assert!(report.failed.is_empty());
    assert!(report.bytes_freed > 0);
    assert!(!p1.exists() && !p2.exists());
}

#[test]
fn delete_mode_defaults_to_trash() {
    assert_eq!(DeleteMode::default(), DeleteMode::Trash);
}
