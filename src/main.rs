// Modules
mod change;
mod cli;
mod core;
mod discover;
mod error;
mod writer;

// Internal imports
use crate::change::Change;
use crate::cli::ColoredOutputMode;
use crate::cli::CommandLineArguments;

// Library imports
use clap::Parser;
use colored::Colorize;
use std::path::Path;
use std::process;

/// Reports the number of changes and unchanged files.
fn print_change_report_and_exit(
    number_of_changed_files: usize,
    number_of_unchanged_files: usize,
    check_only: bool,
) -> ! {
    if check_only && number_of_changed_files > 0 {
        println!("{}", "Oh no! 💥 💔 💥".bold());
    } else {
        println!("{}", "All done! ✨ 🍰 ✨".bold());
    }

    if number_of_changed_files > 0 {
        let message = match (check_only, number_of_changed_files) {
            (true, 1) => "file needs to be formatted.",
            (true, _) => "files need to be formatted.",
            (false, 1) => "file was formatted.",
            (false, _) => "files were formatted.",
        };

        println!(
            "{} {}",
            number_of_changed_files.to_string().blue().bold(),
            message.bold(),
        );
    }

    if number_of_unchanged_files > 0 {
        let message = match (check_only, number_of_unchanged_files) {
            (true, 1) => "file doesn't need to be formatted.",
            (true, _) => "files don't need to be formatted.",
            (false, 1) => "file was left unchanged.",
            (false, _) => "files were left unchanged.",
        };

        print!(
            "{} {}",
            number_of_unchanged_files.to_string().blue(),
            message,
        );
    }

    if check_only && number_of_changed_files > 0 {
        process::exit(1);
    }

    process::exit(0);
}

/// Reports the formatting changes that was made or would be made to a file.
fn print_changes(file_path: &Path, changes: Vec<Change>, check_only: bool) {
    let message = if check_only {
        "needs to be formatted."
    } else {
        "was formatted."
    };
    println!(
        "{} {} {}",
        "File".red().bold(),
        file_path.display().to_string().bold(),
        message.red().bold(),
    );
    for change in changes {
        println!("  ↳ {}", change.to_string(check_only).blue());
    }
}

/// Sets the colored output mode according.
fn set_colored_output_mode(colored_output_mode: &ColoredOutputMode) {
    match colored_output_mode {
        ColoredOutputMode::Auto => { /* Leave it to the colored library. */ }
        ColoredOutputMode::On => colored::control::SHOULD_COLORIZE.set_override(true),
        ColoredOutputMode::Off => colored::control::SHOULD_COLORIZE.set_override(false),
    }
}

/// Command line utility for formatting whitespace in text files.
///
/// It has the following capabilities:
///
///  1. Add a new line marker at the end of the file if it is missing.
///  2. Remove empty lines from the end of the file.
///  3. Remove whitespace from the end of each line.
///  4. Normalize new line markers to Linux, Windows or MacOS.
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
///    whitespace-format \
///         --check-only \
///         --remove-trailing-whitespace \
///         --remove-trailing-empty-lines \
///         --normalize-whitespace-only-files=empty \
///         my_files/
///
///
/// To see all available options, run:
///
///    whitespace-format --help
///
fn main() {
    let command_line_arguments: CommandLineArguments = CommandLineArguments::parse();

    command_line_arguments.validate();

    // Determine whether to use colors or not.
    set_colored_output_mode(&command_line_arguments.color);

    // Compile the regular expression specified by the --exclude command line parameter.
    // Fail early if the expression is invalid.
    let regex = discover::compile_regular_expression(command_line_arguments.exclude.as_str());

    // Discover all files given on the command line.
    let all_files = discover::discover_files(
        &command_line_arguments.paths,
        command_line_arguments.follow_symlinks,
    );

    // Exclude files that match the regular expression specified by the --excluded command line parameter.
    let filtered_files = discover::exclude_files(&all_files, &regex);
    println!("Processing {} file(s)...", filtered_files.len());

    // Process files one by one.
    let mut number_of_changed_files: usize = 0;
    for file_path in &filtered_files {
        let changes = core::process_file(
            file_path,
            &command_line_arguments.get_options(),
            command_line_arguments.check_only,
        );

        if !changes.is_empty() {
            number_of_changed_files += 1;
            print_changes(file_path, changes, command_line_arguments.check_only);
        }
    }

    let number_of_unchanged_files = filtered_files.len() - number_of_changed_files;

    print_change_report_and_exit(
        number_of_changed_files,
        number_of_unchanged_files,
        command_line_arguments.check_only,
    );
}
