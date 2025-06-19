use crate::utils::progress::create_progress_bar;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;

//TODO : Implement the fastest duplicate find function
// Step 1 : Group the files with same size
// Step 2 : Calculate the hash (SHA256) for each file
// Step 3 : Store it in hash map
pub fn get_duplicates(files: &Vec<PathBuf>) -> Vec<PathBuf> {
    let pb = create_progress_bar(&format!("Scanning directory for duplicates"));

    //Step 1 : Group Files by size
    let mut files_by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for file_path in files {
        if !file_path.is_file() || !fs::metadata(file_path).unwrap().is_file() {
            continue;
        }

        files_by_size
            .entry(fs::metadata(file_path).unwrap().len())
            .or_default()
            .push(file_path.clone());
    }

    let mut duplicate_files: Vec<PathBuf> = Vec::new();

    // Step 2 : For files of the same size , group by hash (SHA256 using Sha2 crate)
    // To make it fast use parrellism (rayon crate)
    let potential_duplicate_files: Vec<Vec<PathBuf>> = files_by_size
        .into_par_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .flat_map(|(_, paths)| {
            let mut hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();
            for path in paths {
                if let Ok(hash) = calculate_file_hash(&path) {
                    hashes.entry(hash).or_default().push(path);
                } else {
                    eprintln!("Warning : Couldn't hash file {:?}", path);
                }
            }
            hashes
                .into_values()
                .filter(|group| group.len() > 1)
                .collect::<Vec<Vec<PathBuf>>>()
        })
        .collect();

    println!("\n--- Potential Duplicate Files (grouped by hash) ---");

    if potential_duplicate_files.is_empty() {
        println!("  No duplicate files found based on size and hash.");
    } else {
        for group in potential_duplicate_files {
            duplicate_files.extend_from_slice(&group[1..]);
        }
    }
    pb.finish_with_message("Scan complete for duplicates.");

    duplicate_files
}

fn calculate_file_hash(path: &PathBuf) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192]; // 8KB buffer

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
