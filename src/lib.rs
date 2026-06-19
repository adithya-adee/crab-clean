//! # crab-clean
//!
//! An interactive terminal UI for tidying your file system: find and safely
//! remove **duplicate**, **old**, and **unused** files, with rich filtering and
//! trash-based (recoverable) deletion.
//!
//! The crate is split into a UI-agnostic [`core`] (scanning, detection,
//! filtering, deletion), a [`config`] layer, an [`error`] type, and the
//! ratatui-based [`tui`].
//!
//! ## Example: detect duplicates programmatically
//!
//! ```no_run
//! use crab_clean::core::scanner::{scan, ScanOptions};
//! use crab_clean::core::algorithms::find_duplicates;
//! use std::path::Path;
//!
//! let files = scan(Path::new("."), &ScanOptions::default(), |_| {}).unwrap();
//! let groups = find_duplicates(&files);
//! println!("found {} duplicate group(s)", groups.len());
//! ```

pub mod config;
pub mod core;
pub mod error;
pub mod tui;

pub use config::Config;
pub use error::{CrabError, Result};
