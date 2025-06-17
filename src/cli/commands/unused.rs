use std::path::PathBuf;

use crate::{
    cli::app::interactive_deleting,
    core::{analyzer::get_file_tree, scanner::get_unused}, // error::DeclutterError,
};

pub fn unused_with_dry_run(file_path: String, age: &u32) {
    let files = get_file_tree(&file_path);
    match files {
        Ok(files) => udr(files, age),
        // Err(e) => {
        //     return Err(DeclutterError::Scan(format!(
        //         "Unable to read files in this directory : {}",
        //         e
        //     )));
        Err(_) => println!("Error while reading files"),
    }
}

pub fn udr(files: Vec<PathBuf>, age: &u32) {
    get_unused(&files, age);
}

pub fn unused_with_run(file_path: String, age: &u32) {
    let files = get_file_tree(&file_path);
    match files {
        Ok(files) => ur(files, age),
        // Err(e) => {
        //     return Err(DeclutterError::Scan(format!(
        //         "Unable to read files in this directory : {}",
        //         e
        //     )));
        Err(_) => println!("Error while reading files"),
    }
}

fn ur(files: Vec<PathBuf>, age: &u32) {
    let unused_files = get_unused(&files, age);
    interactive_deleting(unused_files);
}
