//! Terminal user interface built on ratatui + crossterm.

pub mod app;
pub mod theme;
pub mod ui;

use std::time::Duration;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event};

use crate::config::Config;
use crate::error::Result;
use app::App;

/// Run the TUI event loop until the user quits.
///
/// A short poll timeout keeps the UI redrawing while a background scan streams
/// progress, even when the user isn't pressing keys.
pub fn run(terminal: &mut DefaultTerminal, config: Config) -> Result<()> {
    let mut app = App::new(config);

    while app.running {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => app.on_key(key),
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        app.tick();
    }

    Ok(())
}
