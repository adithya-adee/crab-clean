use clap::Parser;

// pub enum CoreOption {

// }

#[derive(Parser, Debug)]
#[command(name = "Pase config")]
#[command(version = "1.0")]
#[command(about = "Parse arguments", long_about = None)]
pub struct Args {
    ///
    #[arg(default_value = "duplicate")]
    pub core_option: String,

    #[arg(default_value = "./fake")]
    pub file_path: String,

    #[arg(default_value = "--dry-run")]
    pub flag: String,
}

pub fn parse_arguments() -> Args {
    let args = Args::parse();

    args
}
