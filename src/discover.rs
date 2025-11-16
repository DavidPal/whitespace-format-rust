// Library imports
use regex::Regex;
use std::fs::DirEntry;
use std::fs::ReadDir;
use std::path::PathBuf;

// Internal imports
use crate::error::die;
use crate::error::Error;

/// Lists all files in a collection of paths (directories or files).
pub fn discover_files(paths: &[PathBuf], follow_symlinks: bool) -> Vec<PathBuf> {
    let mut stack: Vec<PathBuf> = Vec::from(paths);
    let mut files: Vec<PathBuf> = Vec::new();

    while !stack.is_empty() {
        let path = stack.pop().unwrap();

        if !path.exists() {
            die(Error::FileNotFound(path.display().to_string()));
        } else if path.is_symlink() && !follow_symlinks {
            continue;
        } else if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            let inner_paths: ReadDir = path
                .read_dir()
                .unwrap_or_else(|_| die(Error::FailedToReadDirectory(path.display().to_string())));
            for inner_path in inner_paths {
                let inner_path: DirEntry = inner_path.unwrap_or_else(|_| {
                    die(Error::FailedToReadDirectoryEntry(
                        path.display().to_string(),
                    ))
                });
                stack.push(inner_path.path());
            }
        }
    }

    files.sort_unstable();
    files.dedup();
    files
}

/// Compiles regular expression.
pub fn compile_regular_expression(regular_expression: &str) -> Regex {
    if let Ok(regex) = Regex::new(regular_expression) {
        regex
    } else {
        die(Error::InvalidRegularExpression(
            regular_expression.to_string(),
        ));
    }
}

/// Excludes file names that match a regular expression.
pub fn exclude_files(paths: &[PathBuf], regex: &Regex) -> Vec<PathBuf> {
    let mut filtered_files: Vec<PathBuf> = Vec::new();
    for path in paths.iter() {
        if !regex.is_match(path.to_str().unwrap()) {
            filtered_files.push(path.clone());
        }
    }
    filtered_files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::UNMATCHABLE_REGEX;

    #[test]
    fn test_compile_regular_expression() {
        compile_regular_expression("");
        compile_regular_expression(".jpg");
        compile_regular_expression(UNMATCHABLE_REGEX);
    }

    #[test]
    fn test_exclude_files() {
        let regex = compile_regular_expression("\\.(png|jpeg|jpg)$");

        assert_eq!(
            exclude_files(
                &vec![
                    PathBuf::from("photo.jpeg"),
                    PathBuf::from("web_page.html"),
                    PathBuf::from("diagram.png"),
                    PathBuf::from("photo2.jpg"),
                    PathBuf::from("README.txt"),
                    PathBuf::from("Makefile"),
                ],
                &regex
            ),
            vec![
                PathBuf::from("web_page.html"),
                PathBuf::from("README.txt"),
                PathBuf::from("Makefile"),
            ],
        );
    }

    #[test]
    fn test_exclude_files_default() {
        let regex = compile_regular_expression(UNMATCHABLE_REGEX);

        assert_eq!(
            exclude_files(
                &vec![
                    PathBuf::from("photo.jpeg"),
                    PathBuf::from("web_page.html"),
                    PathBuf::from("diagram.png"),
                    PathBuf::from("photo2.jpg"),
                    PathBuf::from("README.txt"),
                    PathBuf::from("Makefile"),
                ],
                &regex
            ),
            vec![
                PathBuf::from("photo.jpeg"),
                PathBuf::from("web_page.html"),
                PathBuf::from("diagram.png"),
                PathBuf::from("photo2.jpg"),
                PathBuf::from("README.txt"),
                PathBuf::from("Makefile"),
            ],
        );
    }

    #[test]
    fn test_discover_files() {
        let files = discover_files(&vec![PathBuf::from("src/")], false);
        assert_eq!(
            files,
            vec![
                PathBuf::from("src/change.rs"),
                PathBuf::from("src/cli.rs"),
                PathBuf::from("src/core.rs"),
                PathBuf::from("src/discover.rs"),
                PathBuf::from("src/error.rs"),
                PathBuf::from("src/main.rs"),
                PathBuf::from("src/writer.rs"),
            ]
        );
    }
}
