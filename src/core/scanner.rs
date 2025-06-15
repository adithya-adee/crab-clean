//TODO : Add pair for interactive delete
use std::path::{Path, PathBuf};

pub fn get_duplicates(files: &Vec<PathBuf>, base_dir_path: &String) -> Vec<PathBuf> {
    if files.len() < 2 {
        println!("Not enough files to find duplicates!!");
    }

    let base_dir = Path::new(base_dir_path);

    let mut duplicate_files: Vec<PathBuf> = Vec::new();

    for i in 2..files.len() {
        for j in i + 1..files.len() {
            let file1 = &files[i];
            let file2 = &files[j];
            let file1 = base_dir.join(&file1);
            let file2 = base_dir.join(&file2);

            if duplicate_files.contains(&file2) {
                continue;
            }
            if compare_file(&file1, &file2) {
                println!("File : {:?} , {:?}", file1, file2);
                if !duplicate_files.contains(&file2) {
                    duplicate_files.push(file2.clone());
                }
            }
        }
    }
    // println!("{:?}", duplicate_files);

    duplicate_files
}

fn compare_file(file1: &PathBuf, file2: &PathBuf) -> bool {
    if !file1.exists() {
        // eprintln!("Warning: File for comparison does not exist: {:?}", file1);
        return false;
    }
    if !file2.exists() {
        // eprintln!("Warning: File for comparison does not exist: {:?}", file2);
        return false;
    }

    let output_res = std::process::Command::new("cmp")
        .arg("--silent")
        .arg(file1)
        .arg(file2)
        .output();

    println!("{:?}", output_res);

    match output_res {
        Ok(output) => {
            // cmp exits with 0 if files are the same, 1 if different, >1 on error.
            // output.status.success() checks for exit code 0.
            output.status.success()
        }
        Err(_e) => false,
    }
}
