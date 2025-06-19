pub mod duplicate;
pub mod unused;

use crate::{
    cli::commands::{
        duplicate::{duplicate_with_dry_run, duplicate_with_run},
        unused::{unused_with_dry_run, unused_with_run},
    },
    config::settings::{Cli, Commands},
    error::CrabcleanError,
};

pub fn dispatch_command(cli: &Cli) -> Result<(), CrabcleanError> {
    match &cli.command {
        Commands::Duplicate { path, dry_run } => {
            if *dry_run {
                println!("Dry run for duplicate command at path: {:?}", path);
                duplicate_with_dry_run(path)?;
            } else if !*dry_run {
                println!("Executing duplicate command at path: {:?}", path);
                duplicate_with_run(path)?;
            } else {
                return Err(CrabcleanError::Config(
                    "Unused command not yet implemented".to_string(),
                ));
            }
        }
        Commands::Unused { path, age, dry_run } => {
            if *dry_run {
                println!(
                    "Unused command with dry run: path={:?}, age={}, dry_run={}",
                    path, age, dry_run
                );
                unused_with_dry_run(path, age)?;
            } else if !*dry_run {
                println!(
                    "Unused command with dry run: path={:?}, age={}, dry_run={}",
                    path, age, dry_run
                );
                unused_with_run(path, age)?;
            } else {
                return Err(CrabcleanError::Config(
                    "Unused command not yet implemented".to_string(),
                ));
            }
        }
    }
    Ok(())
}
