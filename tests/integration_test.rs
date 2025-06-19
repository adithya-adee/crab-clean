use carb_clean::cli::commands::unused::unused_with_dry_run;
use carb_clean::{cli::commands::duplicate::duplicate_with_dry_run, core::scanner::get_file_tree};
use std::fs::remove_dir_all;
use std::io;
use std::{fs, io::Write, path::PathBuf};

#[test]
fn test_get_file_tree() {
    let base_dir_path = PathBuf::from("./src");
    let files = get_file_tree(&base_dir_path);

    if let Some(file) = files.ok() {
        if file.len() > 0 {
            assert!(true)
        }
    } else {
        assert!(false)
    }
}

#[test]
fn test_duplicate_command() -> Result<(), io::Error> {
    let directory_path = "./generate_test_files";
    let generate_dir = generate_directory(directory_path);

    match generate_dir {
        Ok(_) => match duplicate_with_dry_run(&PathBuf::from(directory_path)) {
            Ok(_) => match remove_generate_files(&directory_path) {
                Ok(_) => Ok(()),
                Err(e) => panic!(
                    "Test Duplicate Command Failed by failing to remove the generated directory : {}",
                    e
                ),
            },
            Err(e) => {
                eprintln!("Duplicate ddr : {e}");
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Duplicate dry run failed",
                ))
            }
        },
        Err(e) => panic!("Test Duplicate Command Failed : {}", e),
    }
}

#[test]
fn test_unused_command() -> Result<(), io::Error> {
    let directory_path = "./generate_test_files";
    let generate_dir = generate_directory(directory_path);
    let age: u32 = 1;

    match generate_dir {
        Ok(_) => match unused_with_dry_run(&PathBuf::from(directory_path), &age) {
            Ok(_) => match remove_generate_files(&directory_path) {
                Ok(_) => Ok(()),
                Err(e) => panic!(
                    "Test Duplicate Command Failed by failing to remove the generated directory : {}",
                    e
                ),
            },
            Err(e) => {
                eprintln!("Duplicate ddr : {e}");
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Duplicate dry run failed",
                ))
            }
        },
        Err(e) => panic!("Test Duplicate Command Failed : {}", e),
    }
}

fn generate_directory(directory_path: &str) -> Result<(), io::Error> {
    let _ = fs::create_dir(directory_path);
    let base_file_path = format!("{}/foo.txt", directory_path);
    let mut base_file = fs::File::create(&base_file_path)?;
    base_file.write_all(b"Hello World")?;

    for i in 1..5 {
        let new_file_path = format!("{}/foo{}.txt", directory_path, i);
        fs::copy(&base_file_path, &new_file_path)?;
    }
    Ok(())
}

fn remove_generate_files(dir_path: &'static str) -> Result<(), io::Error> {
    remove_dir_all(dir_path)?;
    Ok(())
}
