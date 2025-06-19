use crate::error::{CrabcleanError, DeclutterResult};
use crate::utils::progress::create_progress_bar;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub fn get_unused(files: &Vec<PathBuf>, age: &u32) -> DeclutterResult<Vec<PathBuf>> {
    if *age == 0 {
        return Err(CrabcleanError::InvalidArgument(
            "Age cannot be zero. Please provide a positive age in days.".to_string(),
        ));
    }

    let pb = create_progress_bar(&format!(
        "Scanning directory for unused items with age: {}",
        age
    ));

    let age_in_days: u64 = (*age).into();
    let duration_ago = Duration::from_secs(age_in_days * 86400);
    let threshold = SystemTime::now()
        .checked_sub(duration_ago)
        .unwrap_or(SystemTime::UNIX_EPOCH);

    println!("Threshold : {:?}", threshold);

    #[cfg(unix)]
    fn get_last_access_time(metadata: &std::fs::Metadata) -> Option<SystemTime> {
        Some(SystemTime::UNIX_EPOCH + Duration::from_secs(metadata.atime() as u64))
    }

    #[cfg(windows)]
    fn get_last_access_time(metadata: &std::fs::Metadata) -> Option<SystemTime> {
        metadata.accessed().ok()
    }

    let unused_files: Vec<PathBuf> = files
        .par_iter()
        .filter_map(|file| match fs::metadata(file) {
            Ok(metadata) => {
                if let Some(accessed) = get_last_access_time(&metadata) {
                    if accessed < threshold {
                        Some(file.clone())
                    } else {
                        None
                    }
                } else {
                    eprintln!(
                        "Warning: Could not retrieve access time for file {:?}",
                        file
                    );
                    None
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: {}. Skipping file {:?}.",
                    CrabcleanError::FileAccess {
                        path: file.clone(),
                        source: e,
                    },
                    file
                );
                None
            }
        })
        .collect();

    println!("No of unused files: {}", unused_files.len());
    pb.finish_with_message("Scan Complete");
    Ok(unused_files)
}
