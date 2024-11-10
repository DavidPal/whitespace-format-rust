// Library imports
use std::process;

pub enum Error {
    FileNotFound(String),
    FailedToReadDirectory(String),
    FailedToReadDirectoryEntry(String),
    FailedToReadFile(String),
    InvalidRegularExpression(String),
}

impl Error {
    pub fn to_string(&self) -> String {
        match &self {
            Error::FileNotFound(message) => format!("File not found {}", message),
            Error::FailedToReadDirectory(message) => {
                format!("Failed to read directory {}", message)
            }
            Error::FailedToReadDirectoryEntry(message) => {
                format!("Failed to read an entry in directory {}", message)
            }
            Error::FailedToReadFile(message) => format!("Failed to read file {}", message),
            Error::InvalidRegularExpression(message) => format!("Failed to read file {}", message),
        }
    }
}

pub fn die(error: Error) -> ! {
    println!("{}", error.to_string());
    process::exit(1);
}
