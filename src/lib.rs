pub mod cli; // Command-line interface handling
pub mod config; // Configuration management
pub mod core; // Core business logic (scanning, analyzing)
pub mod error; // Error types and handling
pub mod utils; // Utility functions

// Re-export commonly used types
pub use config::settings;
pub use error::DeclutterError;
