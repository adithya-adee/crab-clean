pub mod duplicate;
use crate::{
    cli::commands::duplicate::{duplicate_with_dry_run, duplicate_with_run},
    config::settings::{Cli, Commands},
    error::DeclutterError,
};

pub fn dispatch_command(cli: &Cli) -> Result<(), DeclutterError> {
    match &cli.command {
        Commands::Duplicate { path, dry_run, run } => {
            let path_str = path.to_string_lossy().to_string();
            if *dry_run {
                println!("Dry run for duplicate command at path: {:?}", path);
                duplicate_with_dry_run(&path_str);
            } else if *run {
                println!("Executing duplicate command at path: {:?}", path);
                duplicate_with_run(&path_str);
            }
        }
        Commands::Unused { path, age, dry_run } => {
            println!(
                "Unused command: path={:?}, age={}, dry_run={}",
                path, age, dry_run
            );
            return Err(DeclutterError::Config(
                "Unused command not yet implemented".to_string(),
            ));
        }
    }
    Ok(())
}
