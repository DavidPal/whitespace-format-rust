// Internal imports
use super::error::{die, Error};

// Library imports
use regex;
use regex::Regex;
use std::path::PathBuf;

/// Lists all files in a collection of paths (directories or files).
pub fn list_files(paths: &Vec<PathBuf>, follow_symlinks: bool) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = paths.clone();
    let mut files: Vec<PathBuf> = Vec::new();

    loop {
        let mut directories: Vec<PathBuf> = Vec::new();

        for path in paths.iter() {
            if !path.exists() {
                die(Error::FileNotFound(path.display().to_string()))
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
                die(Error::FailedToReadDirectory(
                    directory.display().to_string(),
                ));
            });

            for inner_path in inner_paths {
                let inner_path = inner_path.unwrap_or_else(|_error| {
                    die(Error::FailedToReadDirectory(
                        directory.display().to_string(),
                    ));
                });
                paths.push(inner_path.path());
            }
        }
    }

    return files;
}

pub fn compile_regular_expression(regular_expression: &str) -> Regex {
    if let Ok(regex) = Regex::new(regular_expression) {
        return regex;
    } else {
        die(Error::InvalidRegularExpression(
            regular_expression.to_string(),
        ));
    }
}
