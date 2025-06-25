use crate::{error::CrabcleanError, utils::progress::create_progress_bar};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn get_file_tree(base_dir_path: &PathBuf) -> Result<Vec<PathBuf>, CrabcleanError> {
    let base_dir = Path::new(base_dir_path);

    if !base_dir.is_dir() {
        return Err(CrabcleanError::InvalidArgument(format!(
            "Provided path {:?} is not a directory or does not exist",
            base_dir_path
        )));
    }

    let pb = create_progress_bar(&format!(
        "Scanning files in {}...",
        base_dir_path.to_str().unwrap_or("./")
    ));

    //TODO : Add max depth
    let file_list: Vec<PathBuf> = WalkDir::new(base_dir_path)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| {
            // Skip entries if any component of the path starts with '.'
            !e.path().components().any(|comp| {
                use std::path::Component;
                if let Component::Normal(os_str) = comp {
                    os_str.to_str().map_or(false, |s| s.starts_with('.'))
                } else {
                    false
                }
            })
        })
        .filter_map(|e| e.ok().map(|entry| entry.into_path()))
        .collect();

    pb.finish_with_message("Scan complete.");

    println!("Total Number of Files Scanned : {:?}", file_list.len());

    Ok(file_list)
}
