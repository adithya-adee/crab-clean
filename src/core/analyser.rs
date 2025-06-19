use crate::DeclutterError;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::path::PathBuf;

/// Interactively prompts the user to delete a list of files one by one.
///
/// This function uses `indicatif` for a clean progress bar and standard input
/// for confirmation prompts. It is cross-platform and handles errors gracefully.
///
/// # Arguments
/// * `files`: A slice of `PathBuf` pointing to the files to be considered for deletion.
///
/// # Errors
/// Returns a `DeclutterError` if there's an issue with terminal interaction
/// or if a file deletion fails.
pub fn interactive_deleting(files: &[PathBuf]) -> Result<(), DeclutterError> {
    if files.is_empty() {
        println!("No files to delete.");
        return Ok(());
    }

    println!("\nStarting interactive deletion...");

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{bar:40.cyan/blue} {pos:>7}/{len:7} [{elapsed_precise}]")
            .unwrap()
            .progress_chars("=> "),
    );

    for file in files {
        // Use `pb.println` to print the prompt *above* the progress bar.
        // This keeps the progress bar on the screen without getting garbled.
        if file.is_file() {
            pb.println(format!(
                "Delete '{:?}'? (y/n)",
                file.display() // file.file_name().unwrap_or_default().display() to print full file name
            ));
        } else {
            pb.inc(1);
            continue;
        }

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| DeclutterError::Io(e))?;

        let confirmation = input.trim().eq_ignore_ascii_case("y");

        if confirmation {
            match std::fs::remove_file(file) {
                Ok(_) => {
                    pb.println(format!("✔ Deleted '{}'", file.display()));
                }
                Err(e) => {
                    // On failure, stop everything and report the specific error.
                    pb.finish_with_message("Deletion failed.");
                    return Err(DeclutterError::FileAccess {
                        path: file.clone(),
                        source: e,
                    });
                }
            }
        } else {
            pb.println(format!("✖ Skipped '{}'", file.display()));
        }

        pb.inc(1);
    }

    pb.finish_with_message("Interactive deletion complete.");

    Ok(())
}

pub fn non_interactive_deleting(files: &Vec<PathBuf>) -> Result<(), DeclutterError> {
    if files.is_empty() {
        println!("No files to delete.");
        return Ok(());
    }

    println!("\nStarting non interactive deletion...");

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{bar:40.cyan/blue} {pos:>7}/{len:7} [{elapsed_precise}]")
            .unwrap()
            .progress_chars("=> "),
    );

    files
        .into_par_iter()
        .map(|file| {
            if file.is_file() {
                pb.println(format!("Deleting {}", file.display()));
            } else {
                pb.inc(1);
                return Ok(());
            }

            match std::fs::remove_file(file) {
                Ok(_) => {
                    pb.println(format!("✔ Deleted '{}'", file.display()));
                }
                Err(e) => {
                    // On failure, stop everything and report the specific error.
                    pb.finish_with_message("Deletion failed.");
                    return Err(DeclutterError::FileAccess {
                        path: file.clone(),
                        source: e,
                    });
                }
            }

            pb.inc(1);
            Ok(())
        })
        .collect::<Result<(), DeclutterError>>()?;

    Ok(())
}
