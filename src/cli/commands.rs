pub mod duplicate;
use crate::{
    cli::commands::duplicate::duplicate_with_dry_run, config::settings::Args, error::DeclutterError,
};

pub fn validate_arguments(args: &Args) -> Result<(), DeclutterError> {
    let core_opt = &args.core_option;
    let flag = &args.flag;
    if !validate_core_options(core_opt) {
        return Err(DeclutterError::InvalidArgument(format!(
            "Invalid core option: '{}'",
            core_opt
        )));
    }
    if !validate_flag(flag) {
        return Err(DeclutterError::InvalidArgument(format!(
            "Invalid flag option: '{}'",
            flag
        )));
    }
    Ok(())
}

fn validate_core_options(core_option: &String) -> bool {
    let all_core_opt: Vec<String> = vec!["duplicate".to_string(), "unused".to_string()];
    if all_core_opt.contains(core_option) {
        true
    } else {
        false
    }
}

fn validate_flag(flag: &String) -> bool {
    let all_flags: Vec<String> = vec!["--run".to_string(), "--dry-run".to_string()];

    if all_flags.contains(flag) == true {
        true
    } else {
        false
    }
}

pub fn call_appropriate_command(args: &Args) {
    if args.core_option == String::from("duplicate") {
        if args.flag == String::from("--dry-run") {
            // println!("Call dwdr fn");
            duplicate_with_dry_run(&args.file_path);
        }
    }
}
