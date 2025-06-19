use crate::{CrabcleanError, cli::commands::dispatch_command, config::settings::parse_arguments};

pub fn start_cli() -> Result<(), CrabcleanError> {
    let cli = parse_arguments();
    dispatch_command(&cli)?;
    Ok(())
}
