use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "declutter",
    version = "1.0",
    about = "Declutter your file system"
)]
#[command(propagate_version = true)]
pub struct Cli {
    // TODO : Add optional for Commands
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Duplicate {
        #[arg(default_value = ".")]
        path: PathBuf,

        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    Unused {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long, default_value_t = 30)]
        age: u32,
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
}

pub fn parse_arguments() -> Cli {
    Cli::parse()
}
