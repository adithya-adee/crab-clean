use std::path::PathBuf;

use crate::{
    DeclutterError,
    core::{
        algorithms::duplicate_algo::get_duplicates,
        analyser::{interactive_deleting, non_interactive_deleting},
        scanner::get_file_tree,
    },
};

pub fn duplicate_with_dry_run(path: &PathBuf) -> Result<(), DeclutterError> {
    let files = get_file_tree(path)?;
    let duplicates = get_duplicates(&files);
    let length = &duplicates.len();

    if duplicates.is_empty() {
        println!("No duplicate files found.");
    } else {
        println!("Found the following duplicate files (dry run, no files deleted):");
        println!("Number of Duplicate Files : {}", length);
    }

    Ok(())
}

pub fn duplicate_with_run(path: &PathBuf) -> Result<(), DeclutterError> {
    let files = get_file_tree(path)?;
    let duplicate_files = get_duplicates(&files);

    if duplicate_files.is_empty() {
        println!("No duplicate files found.");
    } else {
        println!("Found the following duplicate files (run with interactive deletion)");
        println!("{}", duplicate_files.len());

        println!("Do you want to enable interactive deleting files ? (y/n)");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| DeclutterError::Io(e))?;

        let confirmation = input.trim().eq_ignore_ascii_case("y");
        if confirmation {
            interactive_deleting(&duplicate_files)?;
        } else {
            non_interactive_deleting(&duplicate_files)?;
        }
    }
    Ok(())
}
