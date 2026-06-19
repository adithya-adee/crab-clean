//! User configuration, persisted as TOML at the platform config dir
//! (e.g. `~/.config/crab-clean/config.toml` on Linux).

use crate::core::delete::DeleteMode;
use crate::core::model::TimeBasis;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Selectable color palette for the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeName {
    #[default]
    Default,
    Ocean,
    Sunset,
    Mono,
}

impl ThemeName {
    pub fn label(&self) -> &'static str {
        match self {
            ThemeName::Default => "Default",
            ThemeName::Ocean => "Ocean",
            ThemeName::Sunset => "Sunset",
            ThemeName::Mono => "Mono",
        }
    }

    /// Cycle to the next palette.
    pub fn next(self) -> Self {
        match self {
            ThemeName::Default => ThemeName::Ocean,
            ThemeName::Ocean => ThemeName::Sunset,
            ThemeName::Sunset => ThemeName::Mono,
            ThemeName::Mono => ThemeName::Default,
        }
    }
}

/// All user-tunable defaults. Everything has a sensible fallback so a missing or
/// partial config file still works (`#[serde(default)]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Directory the home screen starts in.
    pub default_path: String,
    /// Max scan depth (`None` = unlimited).
    pub max_depth: Option<usize>,
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    /// Default age (days) for the unused/old modes.
    pub default_age_days: u32,
    /// Timestamp used for the unused/old comparison and date filters.
    pub time_basis: TimeBasis,
    /// Default deletion behaviour.
    pub delete_mode: DeleteMode,
    pub theme: ThemeName,
    /// Use vim-style navigation (hjkl, gg/G) throughout the UI.
    pub vim_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_path: ".".to_string(),
            max_depth: None,
            include_hidden: false,
            follow_symlinks: false,
            default_age_days: 30,
            time_basis: TimeBasis::default(),
            delete_mode: DeleteMode::default(),
            theme: ThemeName::default(),
            vim_mode: false,
        }
    }
}

impl Config {
    /// Path to the config file (may not exist yet).
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("crab-clean").join("config.toml"))
    }

    /// Load the config, falling back to defaults if it's missing or unreadable.
    pub fn load_or_default() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Persist the current config to disk, creating parent dirs as needed.
    pub fn save(&self) -> Result<()> {
        let path = Self::path().ok_or_else(|| {
            crate::error::CrabError::Config("could not determine config directory".into())
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }
}
