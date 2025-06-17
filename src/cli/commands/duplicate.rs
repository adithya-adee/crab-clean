use std::path::PathBuf;

use crate::{
    cli::app::interactive_deleting,
    core::{analyzer::get_file_tree, scanner::get_duplicates},
};

pub fn duplicate_with_dry_run(file_path: &String) {
    let v = get_file_tree(&file_path);

    match v {
        Ok(v) => ddr(v),
        Err(e) => println!("Failed to get file tree : {:?}", e),
    }
}

fn ddr(files: Vec<PathBuf>) {
    let _duplicate_files = get_duplicates(&files);
    // println!("{:?}", duplicate_files);
}

pub fn duplicate_with_run(file_path: &String) {
    let v = get_file_tree(&file_path);

    match v {
        Ok(v) => dr(v),
        Err(e) => println!("Failed to get file tree : {:?}", e),
    }
}

fn dr(files: Vec<PathBuf>) {
    let duplicate_files = get_duplicates(&files);
    interactive_deleting(duplicate_files);
}
