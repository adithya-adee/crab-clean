use carb_clean::cli::app::start_cli;

fn main() {
    // Start the command line tool here
    if let Err(e) = start_cli() {
        eprintln!("ERROR : {e}");
        std::process::exit(1);
    }
    println!("Declutter CLI finished successfully!!");
}
