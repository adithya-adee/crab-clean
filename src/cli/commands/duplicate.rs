use std::path::PathBuf;

use crate::{
    cli::app::interactive_deleting,
    core::{analyzer::get_file_tree, scanner::get_duplicates},
};

pub fn duplicate_with_dry_run(file_path: &String) {
    let v = get_file_tree(&file_path);

    match v {
        Ok(v) => ddr(v, file_path),
        Err(e) => println!("Failed to get file tree : {:?}", e),
    }
}

fn ddr(files: Vec<PathBuf>, file_path: &String) {
    let duplicate_files = get_duplicates(&files, file_path);
    println!("{:?}", duplicate_files);
    interactive_deleting(duplicate_files, file_path);
}
