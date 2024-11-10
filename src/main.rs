// Modules
mod cli;
mod core;
mod discover;
mod error;
mod writer;

// Library imports
use clap::Parser;
use std::process;

// Internal imports
use crate::cli::CommandLineArguments;

fn file_count(number_of_files: usize) -> String {
    match number_of_files {
        0 => String::new(),
        1 => String::from(format!("{} file", number_of_files)),
        _ => String::from(format!("{} files", number_of_files)),
    }
}

fn print_change_report_and_exit(
    number_of_changed_files: usize,
    number_of_unchanged_files: usize,
    check_only: bool,
) -> ! {
    if check_only && number_of_changed_files > 0 {
        println!("Oh no! 💥 💔 💥");
    } else {
        println!("All done! ✨ 🍰 ✨");
    }

    let check_only_word = if check_only { " would be " } else { " " };

    if number_of_changed_files > 0 {
        print!(
            "{}{}reformatted",
            file_count(number_of_changed_files),
            check_only_word
        );
    }

    if number_of_changed_files > 0 && number_of_unchanged_files > 0 {
        print!(",");
    }

    if number_of_unchanged_files > 0 {
        print!(
            "{}{}left unchanged",
            file_count(number_of_unchanged_files),
            check_only_word
        );
    }
    println!(".");

    if number_of_changed_files > 0 {
        process::exit(1);
    }

    process::exit(0);
}

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

    // Compile the regular expression in the --exclude flag.
    // Fail early if the expression is invalid.
    let regex = discover::compile_regular_expression(command_line_arguments.exclude.as_str());

    // Discover all files given on the command line.
    let files = discover::list_files(
        &command_line_arguments.paths,
        command_line_arguments.follow_symlinks,
    );

    // Exclude files that match the --excluded regular expression.
    let filtered_files = discover::exclude_files(&files, &regex);
    println!("Processing {} file(s)...", filtered_files.len());

    // Process files one by one.
    let mut number_of_changed_files: usize = 0;
    for file in &filtered_files {
        let number_of_changes = core::process_file(
            file,
            &command_line_arguments.get_options(),
            command_line_arguments.check_only,
        );
        if number_of_changes > 0 {
            number_of_changed_files += 1;
        }
    }

    let number_of_unchanged_files = filtered_files.len() - number_of_changed_files;

    print_change_report_and_exit(
        number_of_changed_files,
        number_of_unchanged_files,
        command_line_arguments.check_only,
    );
}
