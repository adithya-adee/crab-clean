//! # Declutter CLI
//!
//! A command-line tool for cleaning up duplicate and unused files.
//!
//! ## Features
//!
//! - Duplicate file detection using SHA-256 hashing
//! - Unused file cleanup based on access time
//! - Interactive deletion with progress tracking
//! - Cross-platform support
//!
//! ## Example
//!
//! ```rust
//! use declutter::core::algorithms::duplicate_algo::get_duplicates;
//! use std::path::PathBuf;
//!
//! let files = vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")];
//! let duplicates = get_duplicates(&files);
//! ```

pub mod cli; // Command-line interface handling
pub mod config; // Configuration management
pub mod core; // Core business logic (scanning, analyzing)
pub mod error; // Error types and handling
pub mod utils; // Utility functions

// Re-export commonly used types
pub use config::settings;
pub use error::DeclutterError;
