use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self},
    path::PathBuf,
};

use crate::CrabcleanError;

pub fn get_group(files: &Vec<PathBuf>, group_by: &String) -> Result<(), CrabcleanError> {
    if group_by == &String::from("extension") {
        group_by_all(files);
    }
    Ok(())
}

fn group_by_all(files: &Vec<PathBuf>) {
    let mut groups: HashMap<&OsStr, Vec<PathBuf>> = HashMap::new();

    for file_path in files {
        if !file_path.is_file() || !fs::metadata(file_path).unwrap().is_file() {
            continue;
        }

        if let Some(extension) = file_path.extension() {
            groups.entry(extension).or_default().push(file_path.clone());
        }
    }

    println!("File Tree by Extension:");

    for (extension, files) in groups.iter() {
        println!("├── {}", extension.to_string_lossy());

        let last_file_index = files.len() - 1;
        for (index, file_path) in files.iter().enumerate() {
            let prefix = if index == last_file_index {
                "└── "
            } else {
                "├── "
            };
            println!(
                "│   {}{}",
                prefix,
                file_path
                    .file_name()
                    .unwrap_or_else(|| OsStr::new(""))
                    .to_string_lossy()
            );
        }
    }
}
