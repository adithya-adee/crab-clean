use console::Term;
use std::{path::PathBuf, process::Command};

pub fn start_cli() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let term = Term::stdout();

    let _ = term.clear_line();
    Ok(())
}

pub fn interactive_deleting(files: Vec<PathBuf>) {
    let term = Term::stdout();

    let pwd = Command::new("pwd").output();
    println!("{:?}", pwd);

    for file in files {
        let file_str = file.to_string_lossy();
        term.write_line(&format!("Delete file '{}'? (y/n):", file_str))
            .unwrap();
        let input = term.read_line().unwrap();
        if input.trim().eq_ignore_ascii_case("y") {
            match Command::new("rm").arg(&file).output() {
                Ok(output) => {
                    if output.status.success() {
                        term.write_line(&format!("Successfully deleted '{}'.", file_str))
                            .unwrap();
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        term.write_line(&format!(
                            "Failed to delete '{}'. Exit code: {:?}, Error: {}",
                            file_str,
                            output.status.code(),
                            stderr
                        ))
                        .unwrap();
                    }
                }
                Err(e) => term
                    .write_line(&format!("Error executing rm for '{}': {}", file_str, e))
                    .unwrap(),
            }
        } else {
            term.write_line(&format!("Skipped '{}'.", file_str))
                .unwrap();
        }
    }
}
