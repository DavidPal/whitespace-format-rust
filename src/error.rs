// Library imports
use colored::Colorize;
use std::fmt;
use std::process;

/// An error.
pub enum Error {
    /// File cannot be found.
    FileNotFound(String),

    /// Directory cannot be read.
    FailedToReadDirectory(String),

    /// Directory entry cannot be read.
    FailedToReadDirectoryEntry(String),

    /// Regular expression (for filtering files) is invalid.
    InvalidRegularExpression(String),

    /// Cannot read file.
    CannotReadFile(String),

    /// Cannot write file.
    CannotWriteFile(String),
}

impl fmt::Display for Error {
    /// Human-readable explanation of an error.
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

/// Prints an error message. The message is printed to standard error output.
pub fn print_error(message: &str) {
    eprintln!("{} {}", "error:".bold().red(), message);
}

/// Prints error message and exits the program.
pub fn die(error: Error) -> ! {
    print_error(&error.to_string());
    process::exit(1);
}
