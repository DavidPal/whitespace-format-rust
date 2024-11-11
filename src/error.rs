// Library imports
use std::process;
use colored::Colorize;

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
                format!("File not found '{}'.", file_path)
            }
            Error::FailedToReadDirectory(directory_path) => {
                format!("Failed to read directory '{}'.", directory_path)
            }
            Error::FailedToReadDirectoryEntry(directory_path) => {
                format!("Failed to read an entry in directory '{}'.", directory_path)
            }
            Error::InvalidRegularExpression(regular_expression) => {
                format!("Failed to read file '{}'.", regular_expression)
            }
            Error::CannotReadFile(file_path) => {
                format!("Cannot open file '{}' for reading.", file_path)
            }
            Error::CannotWriteFile(file_path) => {
                format!("Cannot open file '{}' for writing.", file_path)
            }
        }
    }
}

pub fn die(error: Error) -> ! {
    println!("{}", error.to_string().red().bold());
    process::exit(1);
}

#[allow(dead_code)]
pub trait ErrorReporter {
    fn record(&mut self, error: Error);
    fn report_all_errors(&self);
}

#[allow(dead_code)]
pub struct DyingErrorReporter;

#[allow(dead_code)]
impl DyingErrorReporter {
    pub fn new() -> Self {
        Self {}
    }
}

#[allow(dead_code)]
impl ErrorReporter for DyingErrorReporter {
    fn record(&mut self, error: Error) {
        die(error);
    }
    fn report_all_errors(&self) {
        panic!();
    }
}

#[allow(dead_code)]
pub struct BufferingErrorReporter {
    pub errors: Vec<Error>,
}

#[allow(dead_code)]
impl BufferingErrorReporter {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }
}

#[allow(dead_code)]
impl ErrorReporter for BufferingErrorReporter {
    fn record(&mut self, error: Error) {
        self.errors.push(error);
    }
    fn report_all_errors(&self) {
        for error in &self.errors {
            println!("{}", error.to_string());
        }
    }
}
