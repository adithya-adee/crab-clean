//! crab-clean — interactive TUI entry point.

use crab_clean::config::Config;
use crab_clean::tui;

fn main() {
    let config = Config::load_or_default();

    // Set up the alternate screen + raw mode. `try_init` returns an error
    // (instead of panicking) when there is no interactive terminal.
    let mut terminal = match ratatui::try_init() {
        Ok(terminal) => terminal,
        Err(e) => {
            eprintln!("crab-clean must be run in an interactive terminal: {e}");
            std::process::exit(1);
        }
    };

    let result = tui::run(&mut terminal, config);
    ratatui::restore();

    if let Err(e) = result {
        eprintln!("crab-clean error: {e}");
        std::process::exit(1);
    }
}
