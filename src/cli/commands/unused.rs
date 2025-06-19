use std::path::PathBuf;

use crate::{
    core::{
        algorithms::unused_algo::get_unused,
        analyser::{interactive_deleting, non_interactive_deleting},
        scanner::get_file_tree,
    },
    error::DeclutterError,
};

pub fn unused_with_dry_run(file_path: &PathBuf, age: &u32) -> Result<(), DeclutterError> {
    let files = get_file_tree(&file_path)?;
    let unused_files = get_unused(&files, age)?;

    if unused_files.is_empty() {
        println!("No unused files found");
    } else {
        println!(
            "Number of unused files are : {} which havent been used for {} days",
            unused_files.len(),
            age
        )
    }

    Ok(())
}

pub fn unused_with_run(file_path: &PathBuf, age: &u32) -> Result<(), DeclutterError> {
    let files = get_file_tree(&file_path)?;
    let unused_files = get_unused(&files, age)?;

    if unused_files.is_empty() {
        println!("No duplicate files found.");
    } else {
        println!("Found the following duplicate files (run with interactive deletion)");

        println!("Do you want to enable interactive deleting unused files ? (y/n)");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| DeclutterError::Io(e))?;

        let confirmation = input.trim().eq_ignore_ascii_case("y");
        if confirmation {
            interactive_deleting(&unused_files)?;
        } else {
            non_interactive_deleting(&unused_files)?;
        }
        println!("{}", unused_files.len())
    }

    Ok(())
}
