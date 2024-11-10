// Library imports
use std::process;

pub enum Error {
    FileNotFound(String),
    FailedToReadDirectory(String),
    FailedToReadDirectoryEntry(String),
    FailedToReadFile(String),
    InvalidRegularExpression(String),
    CannotReadFile(String),
    CannotWriteFile(String),
}

impl Error {
    pub fn to_string(&self) -> String {
        match &self {
            Error::FileNotFound(file_path) => format!("File not found {}", file_path),
            Error::FailedToReadDirectory(directory_path) => {
                format!("Failed to read directory {}", directory_path)
            }
            Error::FailedToReadDirectoryEntry(directory_path) => {
                format!("Failed to read an entry in directory {}", directory_path)
            }
            Error::FailedToReadFile(file_path) => format!("Failed to read file {}", file_path),
            Error::InvalidRegularExpression(regular_expression) => {
                format!("Failed to read file {}", regular_expression)
            }
            Error::CannotReadFile(file_path) => {
                format!("Cannot open file {} for reading", file_path)
            }
            Error::CannotWriteFile(file_path) => {
                format!("Cannot open file {} for reading", file_path)
            }
        }
    }
}

pub fn die(error: Error) -> ! {
    println!("{}", error.to_string());
    process::exit(1);
}
