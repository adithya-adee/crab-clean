//! Color palettes for the TUI.

use crate::config::ThemeName;
use ratatui::style::Color;

/// Concrete colors derived from a [`ThemeName`].
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Primary accent (titles, active borders).
    pub accent: Color,
    /// Background of the highlighted (cursor) row.
    pub highlight_bg: Color,
    /// Foreground of the highlighted row.
    pub highlight_fg: Color,
    /// Inactive borders / secondary text.
    pub border: Color,
    /// Marker color for rows selected for deletion.
    pub selected: Color,
    /// Warnings and permanent-delete emphasis.
    pub warning: Color,
    /// Dimmed/auxiliary text.
    pub dim: Color,
}

impl Theme {
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Default => Theme {
                accent: Color::Cyan,
                highlight_bg: Color::Cyan,
                highlight_fg: Color::Black,
                border: Color::DarkGray,
                selected: Color::Green,
                warning: Color::Red,
                dim: Color::Gray,
            },
            ThemeName::Ocean => Theme {
                accent: Color::LightBlue,
                highlight_bg: Color::Blue,
                highlight_fg: Color::White,
                border: Color::DarkGray,
                selected: Color::LightGreen,
                warning: Color::LightRed,
                dim: Color::Gray,
            },
            ThemeName::Sunset => Theme {
                accent: Color::LightMagenta,
                highlight_bg: Color::Magenta,
                highlight_fg: Color::Black,
                border: Color::DarkGray,
                selected: Color::Yellow,
                warning: Color::Red,
                dim: Color::Gray,
            },
            ThemeName::Mono => Theme {
                accent: Color::White,
                highlight_bg: Color::White,
                highlight_fg: Color::Black,
                border: Color::DarkGray,
                selected: Color::White,
                warning: Color::White,
                dim: Color::Gray,
            },
        }
    }
}
