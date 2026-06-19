//! Error types for crab-clean.

use std::path::PathBuf;
use thiserror::Error;

/// The central error type for the whole crate.
#[derive(Debug, Error)]
pub enum CrabError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cannot access '{path}': {source}")]
    FileAccess {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("trash operation failed: {0}")]
    Trash(String),
}

impl From<trash::Error> for CrabError {
    fn from(err: trash::Error) -> Self {
        Self::Trash(err.to_string())
    }
}

impl From<toml::de::Error> for CrabError {
    fn from(err: toml::de::Error) -> Self {
        Self::Config(err.to_string())
    }
}

impl From<toml::ser::Error> for CrabError {
    fn from(err: toml::ser::Error) -> Self {
        Self::Config(err.to_string())
    }
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, CrabError>;
