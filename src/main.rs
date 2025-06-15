use declutter::{
    cli::{
        app::start_cli,
        commands::{call_appropriate_command, validate_arguments},
    },
    config::settings::{Args, parse_arguments},
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

    // Parse the arguments by sending env
    let args: Args = parse_arguments();
    println!("{:?}", args);

    // Validate Arguments
    let validation = validate_arguments(&args);
    match validation {
        Ok(()) => println!("All arguments are valid"),
        Err(e) => {
            eprintln!("Argument validation failed: {}", e);
            std::process::exit(1);
        }
    }

    // Call the appropriate command
    call_appropriate_command(&args);
    println!("Done appropriate command");
}
