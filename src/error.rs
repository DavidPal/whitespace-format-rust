// Library imports
use colored::Colorize;
use std::fmt;
use std::process;

pub enum Error {
    FileNotFound(String),
    FailedToReadDirectory(String),
    FailedToReadDirectoryEntry(String),
    InvalidRegularExpression(String),
    CannotReadFile(String),
    CannotWriteFile(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Error::FileNotFound(file_path) => {
                write!(formatter, "File not found {}.", file_path.bold())
            }
            Error::FailedToReadDirectory(directory_path) => {
                write!(
                    formatter,
                    "Failed to read directory {}.",
                    directory_path.bold()
                )
            }
            Error::FailedToReadDirectoryEntry(directory_path) => {
                write!(
                    formatter,
                    "Failed to read an entry in directory {}.",
                    directory_path.bold()
                )
            }
            Error::InvalidRegularExpression(regular_expression) => {
                write!(
                    formatter,
                    "Failed to read file {}.",
                    regular_expression.bold()
                )
            }
            Error::CannotReadFile(file_path) => {
                write!(formatter, "Cannot read {}", file_path.bold())
            }
            Error::CannotWriteFile(file_path) => {
                write!(formatter, "Cannot write {}", file_path.bold())
            }
        }
    }
}

pub fn print_error(message: &str) {
    eprintln!("{} {}", "error:".bold().red(), message);
}

pub fn die(error: Error) -> ! {
    print_error(&error.to_string());
    process::exit(1);
}
