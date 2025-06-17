use crate::error::DeclutterError;
use indicatif::{ProgressBar, ProgressStyle}; // For spinner
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration, // For spinner
};

pub fn get_file_tree(base_dir_path: &String) -> Result<Vec<PathBuf>, DeclutterError> {
    let base_dir = Path::new(base_dir_path);

    if !base_dir.is_dir() {
        return Err(DeclutterError::InvalidArgument(format!(
            "Provided path {} is not a directory or does not exist",
            base_dir_path
        )));
    }

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    pb.set_message(format!("Scanning files in {}...", base_dir_path));

    //TODO : Add max depth
    let find_files_output = Command::new("find")
        .arg(base_dir_path)
        .arg("-maxdepth")
        .arg("3")
        .arg("-type")
        .arg("f")
        .output()
        .map_err(|err| {
            DeclutterError::InvalidArgument(format!("Failed to execute find: {}", err))
        })?;

    pb.finish_with_message("Scan complete.");

    if !find_files_output.status.success() {
        let stderr = String::from_utf8_lossy(&find_files_output.stderr);
        return Err(DeclutterError::InvalidArgument(format!(
            "Failed to execute find : {}",
            stderr
        )));
    }

    let file_list: Vec<PathBuf> = String::from_utf8_lossy(&find_files_output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();

    println!("Total Number of Files Scanned : {:?}", file_list.len());

    Ok(file_list)
}
