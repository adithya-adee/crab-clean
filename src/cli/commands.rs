pub mod duplicate;
pub mod unused;
use crate::{
    cli::commands::{
        duplicate::{duplicate_with_dry_run, duplicate_with_run},
        unused::{unused_with_dry_run, unused_with_run},
    },
    config::settings::{Cli, Commands},
    error::DeclutterError,
};

pub fn dispatch_command(cli: &Cli) -> Result<(), DeclutterError> {
    match &cli.command {
        Commands::Duplicate { path, run } => {
            let path_str = path.to_string_lossy().to_string();
            if *run {
                println!("Dry run for duplicate command at path: {:?}", path);
                duplicate_with_run(&path_str);
            } else if *run == false {
                println!("Executing duplicate command at path: {:?}", path);
                duplicate_with_dry_run(&path_str);
            } else {
                return Err(DeclutterError::Config(
                    "Unused command not yet implemented".to_string(),
                ));
            }
        }
        Commands::Unused { path, age, run } => {
            let path_str = path.to_string_lossy().to_string();
            if *run {
                println!(
                    "Unused command with dry run: path={:?}, age={}, dry_run={}",
                    path, age, run
                );
                unused_with_run(path_str, age);
            } else if *run == false {
                unused_with_dry_run(path_str, age);
            } else {
                return Err(DeclutterError::Config(
                    "Unused command not yet implemented".to_string(),
                ));
            }
        }
    }
    Ok(())
}
