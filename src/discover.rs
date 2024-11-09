use super::exit::{die,ExitCode};
use std::path::PathBuf;

/// Lists all files in a collection of paths (directories or files).
pub fn list_files(paths: &Vec<PathBuf>, follow_symlinks: bool) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = paths.clone();
    let mut files: Vec<PathBuf> = Vec::new();

    loop {
        let mut directories : Vec<PathBuf> = Vec::new();

        for path in paths.iter() {
            if !path.exists() {
                die(
                    &format!("Path '{}' does not exist.", path.display()),
                    ExitCode::FileNotFound,
                )
            }

            if path.is_symlink() && !follow_symlinks {
                continue;
            }

            if path.is_file() {
                files.push(path.clone());
            }

            if path.is_dir() {
                directories.push(path.clone());
            }
        }

        if directories.is_empty() {
            break;
        }

        paths.clear();

        for directory in directories.iter() {
            let inner_paths = directory.read_dir().unwrap_or_else(|_error| {
                die(
                    &format!("Failed to read directory: {}", directory.display()),
                    ExitCode::FailedToReadDirectory,
                );
            });

            for inner_path in inner_paths {
                let inner_path = inner_path.unwrap_or_else(|_error| {
                    die(
                        &format!("Failed to read an entry in directory: {}", directory.display()),
                        ExitCode::FailedToReadDirectoryEntry,
                    );
                });
                paths.push(inner_path.path());
            }
        }
    }

    return files;
}
