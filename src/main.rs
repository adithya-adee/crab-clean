use declutter::{
    cli::{app::start_cli, commands::dispatch_command},
    config::settings::{Cli, parse_arguments},
};

fn main() {
    // Start the command line tool here
    let start = start_cli();

    match start {
        Ok(()) => println!("CLI Started!!"),
        Err(_) => {
            panic!("Error starting cli");
        }
    }

    // Parse the arguments
    let cli_args: Cli = parse_arguments();
    println!("{:?}", cli_args);

    // Call the appropriate command
    if let Err(e) = dispatch_command(&cli_args) {
        eprintln!("Error executing command: {}", e);
        std::process::exit(1);
    }
    println!("Command finished.");
}
