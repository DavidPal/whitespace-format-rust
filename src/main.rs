#![allow(dead_code)]

// Modules
mod cli;
mod core;
mod discover;
mod error;
mod writer;

// Library imports
use clap::Parser;
use std::fs;

// Internal imports
use cli::CommandLineArguments;
use core::Change;

const FILE_NAME: &str = "README.md";

/// Command line utility for formatting whitespace in text files.
///
/// It has the following capabilities:
///
///  1. Add a new line marker at the end of the file if it is missing.
///  2. Remove empty lines from the end of the file.
///  3. Remove whitespace from the end of each line.
///  4. Normalize new line markers to Linux, MacOS, or Windows.
///  4. Replace tabs with spaces.
///  5. Replace files consisting of only whitespace with zero bytes.
///
/// The program automatically detects line endings used in the files.
///
/// The program reports any changes made to the files.
///
/// With the `--check-only` option, the program reports if files would be changed,
/// without changing them.
///
/// Sample usage:
///
/// TODO
///
fn main() {
    let command_line_arguments: CommandLineArguments = CommandLineArguments::parse();
    dbg!(&command_line_arguments);

    let regex = discover::compile_regular_expression(command_line_arguments.exclude.as_str());
    let files = discover::list_files(
        &command_line_arguments.paths,
        command_line_arguments.follow_symlinks,
    );
    let _filtered_files = discover::exclude_files(&files, &regex);

    // Print content of a file.
    let input_data: Vec<u8> = fs::read(&FILE_NAME).unwrap();
    dbg!(String::from_utf8_lossy(&input_data));

    let (output_data, changes): (Vec<u8>, Vec<Change>) =
        core::process_file(&input_data, &command_line_arguments.get_options());

    println!("Number of changes {}", changes.len());
    for change in changes {
        println!("Line {}: {}", change.line_number, change.change_type);
    }

    dbg!(String::from_utf8_lossy(&output_data));
}
