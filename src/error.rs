use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CrabcleanError {
    Io(io::Error),
    FileAccess { path: PathBuf, source: io::Error },
    Config(String),
    Scan(String),
    InvalidArgument(String),
}

impl std::error::Error for CrabcleanError {}

impl fmt::Display for CrabcleanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::FileAccess { path, source } => {
                write!(f, "Cannot access file '{}': {}", path.display(), source)
            }
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::Scan(msg) => write!(f, "File scan error: {}", msg),
            Self::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
        }
    }
}

impl From<io::Error> for CrabcleanError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

pub type CrabCleanResult<T> = Result<T, CrabcleanError>;
