// Library imports
use colored::Colorize;
use std::process;

pub enum Error {
    FileNotFound(String),
    FailedToReadDirectory(String),
    FailedToReadDirectoryEntry(String),
    InvalidRegularExpression(String),
    CannotReadFile(String),
    CannotWriteFile(String),
}

impl Error {
    pub fn to_string(&self) -> String {
        match &self {
            Error::FileNotFound(file_path) => {
                format!("File not found {}.", file_path.bold())
            }
            Error::FailedToReadDirectory(directory_path) => {
                format!("Failed to read directory {}.", directory_path.bold())
            }
            Error::FailedToReadDirectoryEntry(directory_path) => {
                format!(
                    "Failed to read an entry in directory {}.",
                    directory_path.bold()
                )
            }
            Error::InvalidRegularExpression(regular_expression) => {
                format!("Failed to read file {}.", regular_expression.bold())
            }
            Error::CannotReadFile(file_path) => {
                format!("Cannot read {}", file_path.bold())
            }
            Error::CannotWriteFile(file_path) => {
                format!("Cannot write {}", file_path.bold())
            }
        }
    }
}

pub fn die(error: Error) -> ! {
    println!("{}", error.to_string().red().bold());
    process::exit(1);
}
