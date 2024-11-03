use crate::ChangeType::ReplacedWhiteSpaceOnlyFileWithOneLine;
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
use std::env;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process;

const FILE_NAME: &str = "README.md";

// ASCII codes of characters that we care about.
// For efficiency, we encode the characters as unsigned bytes.
// This way we avoid Unicode character decoding and encoding.
const CARRIAGE_RETURN: u8 = b'\r';
const LINE_FEED: u8 = b'\n';
const SPACE: u8 = b' ';
const TAB: u8 = b'\t';
const VERTICAL_TAB: u8 = 0x0B; // The same as '\v' in C, C++, Java and Python.
const FORM_FEED: u8 = 0x0C; // The same as '\f' in C, C++, Java and Python.

enum ExitCode {
    FileNotFound = 1,
    FailedToReadDirectory = 2,
    FailedToReadDirectoryEntry = 3,
    FailedToReadFile = 4,
}

/// A possible line ending.
#[derive(PartialEq, Debug)]
enum NewLineMarker {
    // Linux line ending is a single line feed character '\n'.
    Linux,

    // MacOS line ending is a single carriage return character '\r'.
    MacOs,

    // Windows/DOS line ending is a sequence of two characters:
    // carriage return character followed by line feed character.
    Windows,
}

impl NewLineMarker {
    fn to_bytes(&self) -> &'static [u8] {
        match &self {
            NewLineMarker::Linux => &[LINE_FEED],
            NewLineMarker::MacOs => &[CARRIAGE_RETURN],
            NewLineMarker::Windows => &[CARRIAGE_RETURN, LINE_FEED],
        }
    }
}

/// New line marker that should be used in the output files.
#[derive(PartialEq, Debug)]
enum OutputNewLineMarkerMode {
    Auto,
    Linux,
    MacOs,
    Windows,
}

#[derive(PartialEq, Debug)]
enum NonStandardWhitespaceReplacementMode {
    Ignore,
    ReplaceWithSpace,
    Remove,
}

#[derive(PartialEq, Debug)]
enum TrivialFileReplacementMode {
    Ignore,
    Empty,
    OneLine,
}

struct Options {
    add_new_line_marker_at_end_of_file: bool,
    remove_new_line_marker_from_end_of_file: bool,
    normalize_new_line_markers: bool,
    remove_trailing_whitespace: bool,
    remove_trailing_empty_lines: bool,
    new_line_marker: OutputNewLineMarkerMode,
    normalize_empty_files: TrivialFileReplacementMode,
    normalize_whitespace_only_files: TrivialFileReplacementMode,
    replace_tabs_with_spaces: isize,
    normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode,
}

#[derive(PartialEq, Debug)]
enum ChangeType {
    NewLineMarkerAddedToEndOfFile,
    NewLineMarkerRemovedFromEndOfFile,
    ReplacedNewLineMarker,
    RemovedTrailingWhitespace,
    RemovedEmptyLines,
    ReplacedEmptyFileWithOneLine,
    ReplacedWhiteSpaceOnlyFileWithEmptyFile,
    ReplacedWhiteSpaceOnlyFileWithOneLine,
    ReplacedTabWithSpaces,
    RemovedTab,
    ReplacedNonstandardWhitespaceWithSpace,
    RemovedNonstandardWhitespace,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printable = match *self {
            Self::NewLineMarkerAddedToEndOfFile => "NewLineMarkerAddedToEndOfFile",
            Self::NewLineMarkerRemovedFromEndOfFile => "NewLineMarkerRemovedFromEndOfFile",
            Self::ReplacedNewLineMarker => "ReplacedNewLineMarker",
            Self::RemovedTrailingWhitespace => "RemovedTrailingWhitespace",
            Self::RemovedEmptyLines => "RemovedEmptyLines",
            Self::ReplacedEmptyFileWithOneLine => "ReplacedEmptyFileWithOneLine",
            Self::ReplacedWhiteSpaceOnlyFileWithEmptyFile => {
                "ReplacedWhiteSpaceOnlyFileWithEmptyFile"
            }
            Self::ReplacedWhiteSpaceOnlyFileWithOneLine => "ReplacedWhiteSpaceOnlyFileWithOneLine",
            Self::ReplacedTabWithSpaces => "ReplacedTabWithSpaces",
            Self::RemovedTab => "RemovedTab",
            Self::ReplacedNonstandardWhitespaceWithSpace => {
                "ReplacedNonstandardWhitespaceWithSpace"
            }
            Self::RemovedNonstandardWhitespace => "RemovedNonstandardWhitespace",
        };
        write!(f, "{:?}", printable)
    }
}

struct Change {
    line_number: usize,
    change_type: ChangeType,
}

fn is_file_whitespace(input_data: &[u8]) -> bool {
    for char in input_data {
        match *char {
            CARRIAGE_RETURN => continue,
            LINE_FEED => continue,
            SPACE => continue,
            TAB => continue,
            VERTICAL_TAB => continue,
            FORM_FEED => continue,
            _ => return false,
        };
    }
    return true;
}

fn process_file(input_data: &[u8], options: &Options) -> (Vec<u8>, Vec<Change>) {
    // Figure out what new line marker to use when writing to the output buffer.
    let output_new_line_marker = match options.new_line_marker {
        OutputNewLineMarkerMode::Auto => find_most_common_new_line_marker(input_data),
        OutputNewLineMarkerMode::Linux => NewLineMarker::Linux,
        OutputNewLineMarkerMode::MacOs => NewLineMarker::MacOs,
        OutputNewLineMarkerMode::Windows => NewLineMarker::Windows,
    };

    // Handle empty file.
    if input_data.len() == 0 {
        return match options.normalize_empty_files {
            TrivialFileReplacementMode::Empty | TrivialFileReplacementMode::Ignore => {
                (Vec::new(), Vec::new())
            }
            TrivialFileReplacementMode::OneLine => (
                Vec::from(output_new_line_marker.to_bytes()),
                Vec::from([Change {
                    line_number: 1,
                    change_type: ChangeType::ReplacedEmptyFileWithOneLine,
                }]),
            ),
        };
    }

    // Handle non-empty file consisting of whitespace only.
    if is_file_whitespace(input_data) {
        return match options.normalize_whitespace_only_files {
            TrivialFileReplacementMode::Empty => (
                Vec::new(),
                Vec::from([Change {
                    line_number: 1,
                    change_type: ChangeType::ReplacedWhiteSpaceOnlyFileWithEmptyFile,
                }]),
            ),
            TrivialFileReplacementMode::Ignore => (Vec::from(input_data), Vec::new()),
            TrivialFileReplacementMode::OneLine => {
                if input_data == output_new_line_marker.to_bytes() {
                    (Vec::from(output_new_line_marker.to_bytes()), Vec::new())
                } else {
                    (
                        Vec::from(output_new_line_marker.to_bytes()),
                        Vec::from([Change {
                            line_number: 1,
                            change_type: ChangeType::ReplacedWhiteSpaceOnlyFileWithOneLine,
                        }]),
                    )
                }
            }
        };
    }

    // Index into the input buffer.
    let mut i: usize = 0;

    // The output buffer.
    // The output is expected to be about the same size as the input.
    // If there are no changes, the output will be exactly the same as input.
    // In order to avoid too many reallocations, reserve the capacity
    // equal to the size of the input buffer.
    let mut output_data: Vec<u8> = Vec::with_capacity(input_data.len());

    // List of changes between input and output.
    let mut changes: Vec<Change> = Vec::new();

    // Line number. It is incremented every time we encounter a new end of line marker.
    let mut line_number: usize = 1;

    // Position one byte past the end of last line in the output buffer
    // excluding the last end of line marker.
    let mut last_end_of_line_excluding_eol_marker: usize = 0;

    // Position one byte past the end of last line in the output buffer
    // including the last end of line marker.
    let mut last_end_of_line_including_eol_marker: usize = 0;

    // Position one byte past the last non-whitespace character in the output buffer.
    let mut last_non_whitespace: usize = 0;

    // Position one byte past the end of last non-empty line in the output buffer
    // excluding the last end of line marker.
    let mut last_end_of_non_empty_line_excluding_eol_marker: usize = 0;

    // Position one byte past the end of last non-empty line in the output buffer,
    // including the last end of line marker.
    let mut last_end_of_non_empty_line_including_eol_marker: usize = 0;

    // Line number of the last non-empty line.
    let mut last_non_empty_line_number: usize = 0;

    while i < input_data.len() {
        if input_data[i] == CARRIAGE_RETURN || input_data[i] == LINE_FEED {
            // Parse the new line marker
            let new_line_marker: NewLineMarker;
            if input_data[i] == LINE_FEED {
                new_line_marker = NewLineMarker::Linux;
            } else if i < input_data.len() - 1 && input_data[i + 1] == LINE_FEED {
                new_line_marker = NewLineMarker::Windows;
                // Windows new line marker consists of two bytes.
                // Skip the extra byte.
                i += 1;
            } else {
                new_line_marker = NewLineMarker::MacOs;
            }

            // Remove trailing whitespace
            if options.remove_trailing_whitespace && last_non_whitespace < output_data.len() {
                changes.push(Change {
                    line_number: line_number,
                    change_type: ChangeType::RemovedTrailingWhitespace,
                });
                output_data.truncate(last_non_whitespace);
            }

            // Determine if the last line is empty
            let is_empty_line: bool = last_end_of_line_including_eol_marker == output_data.len();

            // Add new line marker
            last_end_of_line_excluding_eol_marker = output_data.len();
            if options.normalize_new_line_markers && output_new_line_marker != new_line_marker {
                changes.push(Change {
                    line_number: line_number,
                    change_type: ChangeType::ReplacedNewLineMarker,
                });
                output_data.extend_from_slice(output_new_line_marker.to_bytes());
            } else {
                output_data.extend_from_slice(output_new_line_marker.to_bytes());
            }
            last_end_of_line_including_eol_marker = output_data.len();

            // Update position of last non-empty line.
            if !is_empty_line {
                last_end_of_non_empty_line_excluding_eol_marker =
                    last_end_of_line_excluding_eol_marker;
                last_end_of_non_empty_line_including_eol_marker =
                    last_end_of_line_including_eol_marker;
                last_non_empty_line_number = line_number;
            }
            line_number += 1;
        } else if input_data[i] == SPACE {
            output_data.push(input_data[i]);
        } else if input_data[i] == TAB {
            if options.replace_tabs_with_spaces < 0 {
                output_data.push(input_data[i]);
            } else if options.replace_tabs_with_spaces > 0 {
                changes.push(Change {
                    line_number: line_number,
                    change_type: ChangeType::ReplacedTabWithSpaces,
                });
                for _ in 0..options.replace_tabs_with_spaces {
                    output_data.push(SPACE);
                }
            } else {
                // Remove the tab character.
                changes.push(Change {
                    line_number: line_number,
                    change_type: ChangeType::RemovedTab,
                });
            }
        } else if input_data[i] == VERTICAL_TAB || input_data[i] == FORM_FEED {
            match options.normalize_non_standard_whitespace {
                NonStandardWhitespaceReplacementMode::Ignore => {
                    output_data.push(input_data[i]);
                }
                NonStandardWhitespaceReplacementMode::ReplaceWithSpace => {
                    output_data.push(SPACE);
                    changes.push(Change {
                        line_number: line_number,
                        change_type: ChangeType::ReplacedNonstandardWhitespaceWithSpace,
                    });
                }
                NonStandardWhitespaceReplacementMode::Remove => {
                    // Remove the non-standard whitespace character.
                    changes.push(Change {
                        line_number: line_number,
                        change_type: ChangeType::RemovedNonstandardWhitespace,
                    });
                }
            }
        } else {
            output_data.push(input_data[i]);
            last_non_whitespace = output_data.len();
        }

        // Move to the next byte
        i += 1;
    }

    // Remove trailing whitespace from the last line.
    if options.remove_trailing_whitespace
        && last_end_of_line_including_eol_marker < output_data.len()
        && last_non_whitespace < output_data.len()
    {
        changes.push(Change {
            line_number: line_number,
            change_type: ChangeType::RemovedTrailingWhitespace,
        });
        output_data.truncate(last_non_whitespace);
    }

    // Remove trailing empty lines.
    if options.remove_trailing_empty_lines
        && last_end_of_line_including_eol_marker == output_data.len()
        && last_end_of_non_empty_line_including_eol_marker < output_data.len()
    {
        line_number = last_non_empty_line_number + 1;
        last_end_of_line_excluding_eol_marker = last_end_of_non_empty_line_excluding_eol_marker;
        last_end_of_line_including_eol_marker = last_end_of_non_empty_line_including_eol_marker;
        changes.push(Change {
            line_number: line_number,
            change_type: ChangeType::RemovedEmptyLines,
        });
        output_data.truncate(last_end_of_non_empty_line_including_eol_marker);
    }

    // Add new line marker at the end of the file
    if options.add_new_line_marker_at_end_of_file
        && last_end_of_line_including_eol_marker < output_data.len()
    {
        last_end_of_line_excluding_eol_marker = output_data.len();
        changes.push(Change {
            line_number: line_number,
            change_type: ChangeType::NewLineMarkerAddedToEndOfFile,
        });
        output_data.extend_from_slice(output_new_line_marker.to_bytes());
        last_end_of_line_including_eol_marker = output_data.len();
        line_number += 1;
    }

    // Remove new line marker from the end of the file
    if options.remove_new_line_marker_from_end_of_file
        && last_end_of_line_including_eol_marker == output_data.len()
        && line_number >= 2
    {
        line_number -= 1;
        changes.push(Change {
            line_number: line_number,
            change_type: ChangeType::NewLineMarkerRemovedFromEndOfFile,
        });
        output_data.truncate(last_end_of_line_excluding_eol_marker);
    }

    return (output_data, changes);
}

/// Computes the most common new line marker based on content of the file.
/// If there are ties, prefer Linux to MacOS to Windows.
/// If there are no new line markers, return Linux.
fn find_most_common_new_line_marker(input: &[u8]) -> NewLineMarker {
    let mut linux_count: usize = 0;
    let mut macos_count: usize = 0;
    let mut windows_count: usize = 0;
    let mut i: usize = 0;

    while i < input.len() {
        if input[i] == CARRIAGE_RETURN {
            if i < input.len() - 1 && input[i + 1] == LINE_FEED {
                windows_count += 1;
                i += 1;
            } else {
                macos_count += 1;
            }
        } else if input[i] == LINE_FEED {
            linux_count += 1;
        }
        i += 1;
    }

    if linux_count >= macos_count && linux_count >= windows_count {
        return NewLineMarker::Linux;
    } else if macos_count >= windows_count {
        return NewLineMarker::MacOs;
    }
    return NewLineMarker::Windows;
}

fn die(message: &str, exit_code: ExitCode) -> ! {
    println!("{}", message);
    process::exit(exit_code as i32);
}

fn list_files(path: PathBuf, follow_symlinks: bool) -> Vec<PathBuf> {
    if !path.exists() {
        die(
            &format!("Path '{}' does not exist.", path.display()),
            ExitCode::FileNotFound,
        )
    }

    if !follow_symlinks && path.is_symlink() {
        return Vec::new();
    }

    if path.is_file() {
        return vec![path];
    }

    if path.is_dir() {
        let inner_paths = path.read_dir().unwrap_or_else(|error| {
            die(
                &format!("Failed to read directory: {}", path.display()),
                ExitCode::FailedToReadDirectory,
            );
        });

        let mut files: Vec<PathBuf> = Vec::new();
        for inner_path in inner_paths {
            let inner_path = inner_path.unwrap_or_else(|error| {
                die(
                    &format!("Failed to read entry in directory: {}", path.display()),
                    ExitCode::FailedToReadDirectory,
                );
            });
            files.extend(list_files(inner_path.path(), follow_symlinks));
        }
        return files;
    }

    return Vec::new();
}

fn main() {
    // Print hello world.
    println!("Hello, world!");

    // Print command line arguments.
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    // Print content of a file.
    let input_data: Vec<u8> = fs::read(&FILE_NAME).unwrap();
    dbg!(String::from_utf8_lossy(&input_data));

    let new_line_marker: NewLineMarker = find_most_common_new_line_marker(&input_data);
    println!("Most common new line marker is {:?}", new_line_marker);

    let options: Options = Options {
        add_new_line_marker_at_end_of_file: true,
        remove_new_line_marker_from_end_of_file: false,
        normalize_new_line_markers: true,
        remove_trailing_whitespace: true,
        remove_trailing_empty_lines: true,
        new_line_marker: OutputNewLineMarkerMode::Linux,
        normalize_empty_files: TrivialFileReplacementMode::Empty,
        normalize_whitespace_only_files: TrivialFileReplacementMode::Empty,
        replace_tabs_with_spaces: -1,
        normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode::Ignore,
    };
    let (output_data, changes): (Vec<u8>, Vec<Change>) = process_file(&input_data, &options);

    println!("Number of changes {}", changes.len());
    for change in changes {
        println!("Line {}: {}", change.line_number, change.change_type);
    }

    dbg!(String::from_utf8_lossy(&output_data));

    println!("{:?}", list_files(PathBuf::from("./"), false));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_whitespace() {
        assert_eq!(is_file_whitespace(&[]), true);
        assert_eq!(is_file_whitespace(b" \t\n\r"), true);
        assert_eq!(is_file_whitespace(b"hello"), false);
        assert_eq!(is_file_whitespace(b"hello world\n"), false);
    }

    #[test]
    fn test_find_most_common_new_line_marker() {
        assert_eq!(find_most_common_new_line_marker(&[]), NewLineMarker::Linux);
        assert_eq!(
            find_most_common_new_line_marker(&[LINE_FEED]),
            NewLineMarker::Linux
        );
        assert_eq!(
            find_most_common_new_line_marker(&[CARRIAGE_RETURN]),
            NewLineMarker::MacOs
        );
        assert_eq!(
            find_most_common_new_line_marker(&[CARRIAGE_RETURN, LINE_FEED]),
            NewLineMarker::Windows
        );
        assert_eq!(
            find_most_common_new_line_marker(b"hello world"),
            NewLineMarker::Linux
        );
        assert_eq!(
            find_most_common_new_line_marker(b"a\rb\nc\n"),
            NewLineMarker::Linux
        );
        assert_eq!(
            find_most_common_new_line_marker(b"a\rb\rc\r\n"),
            NewLineMarker::MacOs
        );
        assert_eq!(
            find_most_common_new_line_marker(b"a\r\nb\r\nc\n"),
            NewLineMarker::Windows
        );
    }

    #[test]
    fn test_process_file() {
        let options: Options = Options {
            add_new_line_marker_at_end_of_file: true,
            remove_new_line_marker_from_end_of_file: false,
            normalize_new_line_markers: true,
            remove_trailing_whitespace: true,
            remove_trailing_empty_lines: true,
            new_line_marker: OutputNewLineMarkerMode::Linux,
            normalize_empty_files: TrivialFileReplacementMode::Empty,
            normalize_whitespace_only_files: TrivialFileReplacementMode::Empty,
            replace_tabs_with_spaces: -1,
            normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode::Ignore,
        };
        assert_eq!(process_file(b"hello world", &options).0, b"hello world\n");
    }

    #[test]
    fn test_process_file_2() {
        let options: Options = Options {
            add_new_line_marker_at_end_of_file: true,
            remove_new_line_marker_from_end_of_file: false,
            normalize_new_line_markers: false,
            remove_trailing_whitespace: true,
            remove_trailing_empty_lines: true,
            new_line_marker: OutputNewLineMarkerMode::Linux,
            normalize_empty_files: TrivialFileReplacementMode::Empty,
            normalize_whitespace_only_files: TrivialFileReplacementMode::Empty,
            replace_tabs_with_spaces: -1,
            normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode::Ignore,
        };
        assert_eq!(
            process_file(b"hello world\r\n", &options).0,
            b"hello world\n"
        );
    }
}
