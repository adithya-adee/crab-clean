use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "crabclean",
    version = "0.1.0",
    about = "Clean up your file system",
    long_about = "
Clean up your file system by finding and managing duplicate and unused files.

USAGE:
    crabclean <COMMAND> [OPTIONS] [PATH]

COMMANDS:
    duplicate    Find and manage duplicate files
    unused       Find and manage unused files
    help         Print this message or the help of the given subcommand(s)

FLAGS:
    --dry-run, -n   Preview what would be deleted without actually deleting files.
                    If omitted, you will be interactively prompted for each deletion.
                    (Press Ctrl+C to exit at any prompt.)

OPTIONS:
    -a, --age <AGE>   (for 'unused' command) Age in days for a file to be considered unused (default: 30)
    -h, --help        Print help information
    -V, --version     Print version information

EXAMPLES:
    crabclean duplicate --dry-run
    crabclean duplicate /path/to/dir --dry-run
    crabclean duplicate ~/Downloads
    crabclean unused ~/Documents --age 60 --dry-run
    crabclean unused --age 90

NOTES:
    - If you omit the --dry-run flag, the tool will prompt you for each file before deletion.
    - Use Ctrl+C at any prompt to safely exit without deleting files.
    - For feature requests (like grouping files) or issues, please contact: your.email@example.com
"
)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Find and manage duplicate files",
        long_about = "Finds duplicate files in the specified directory using SHA-256 content hashing. Supports dry run and interactive deletion."
    )]
    Duplicate {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    #[command(
        about = "Find and manage unused files",
        long_about = "Finds files that have not been accessed for a specified number of days. Supports dry run and interactive deletion."
    )]
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
