use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::DeclutterError;

pub fn get_file_tree(base_dir_path: &String) -> Result<Vec<PathBuf>, DeclutterError> {
    let base_dir = Path::new(base_dir_path);

    if !base_dir.is_dir() {
        return Err(DeclutterError::InvalidArgument(format!(
            "Provided path {} is not a directory or does not exist",
            base_dir_path
        )));
    }

    let find_files = Command::new("ls")
        .arg(base_dir_path)
        .arg("-f")
        .output()
        .map_err(|err| {
            DeclutterError::InvalidArgument(format!("Failed to execute find: {}", err))
        })?;

    if !find_files.status.success() {
        let stderr = String::from_utf8_lossy(&find_files.stderr);
        return Err(DeclutterError::InvalidArgument(format!(
            "Failed to execute ls : {}",
            stderr
        )));
    }

    let file_list: Vec<PathBuf> = String::from_utf8_lossy(&find_files.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();

    Ok(file_list)
}
