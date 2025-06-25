use std::path::PathBuf;

use crate::{
    core::{algorithms::group_algo::get_group, scanner::get_file_tree},
    error::CrabcleanError,
};

pub fn group_by_dry_run(file_path: &PathBuf, group_by: &String) -> Result<(), CrabcleanError> {
    let files = get_file_tree(&file_path)?;
    println!("{:?}", files);
    let _unused_files = get_group(&files, group_by)?;

    // if unused_files.is_empty() {
    //     println!("No unused files found");
    // } else {
    //     println!(
    //         "Number of unused files are : {} which havent been used for {} days",
    //         unused_files.len(),
    //         group_by
    //     )
    // }

    Ok(())
}

pub fn group_by_run(file_path: &PathBuf, group_by: &String) -> Result<(), CrabcleanError> {
    let files = get_file_tree(&file_path)?;
    let _unused_files = get_group(&files, group_by)?;

    // if unused_files.is_empty() {
    //     println!("No duplicate files found.");
    // } else {
    //     println!("Found the following duplicate files (run with interactive deletion)");

    //     println!("Do you want to enable interactive deleting unused files ? (y/n)");
    //     let mut input = String::new();
    //     std::io::stdin()
    //         .read_line(&mut input)
    //         .map_err(|e| CrabcleanError::Io(e))?;

    //     let confirmation = input.trim().eq_ignore_ascii_case("y");
    //     if confirmation {
    //         interactive_deleting(&unused_files)?;
    //     } else {
    //         non_interactive_deleting(&unused_files)?;
    //     }
    //     println!("{}", unused_files.len())
    // }

    Ok(())
}
