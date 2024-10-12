// This program formats whitespace in text files.
// It can
//  1. Ensure the consistent line endings (Linux, MacOS, Windows).
//  2. Ensure that each text file ends with a new line marker.
//  3. Remove empty lines from the end of the file.
//  4. Remove whitespace from the end of each line.
//  5. Replace tabs with spaces.
//  6. Ensure that empty files have zero bytes.

use std::env;
use std::fs;
use std::fmt;

const FILE_NAME: &str = "README.md";

// ASCII codes of characters that we care about.
// For efficiency, we encode the characters as unsigned bytes.
// This way we avoid Unicode character decoding and encoding.
const CARRIAGE_RETURN: u8 = b'\r';
const LINE_FEED: u8 = b'\n';
const SPACE: u8 = b' ';
const TAB: u8 = b'\t';
const VERTICAL_TAB: u8 = 0x0B; // The same as '\v' in C, C++, Java and Python.
const FORM_FEED: u8 = 0x0C;  // The same as '\f' in C, C++, Java and Python.

// Possible line ending.
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

#[derive(PartialEq)]
enum NewLineMarkerMode {
    Auto,
    Linux,
    MacOs,
    Windows,
}

enum NonStandardWhitespaceReplacementMode {
    Ignore,
    ReplaceWithSpace,
    Remove,
}

enum EmptyFileReplacementMode {
    Ignore,
    Empty,
    OneLine,
}

impl fmt::Display for NewLineMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printable = match *self {
            Self::Linux => "Linux",
            Self::MacOs => "MacOS",
            Self::Windows => "Windows",
        };
        write!(f, "{:?}", printable)
    }
}

struct Options {
    add_new_line_marker_at_end_of_file: bool,
    remove_new_line_marker_from_end_of_file: bool,
    normalize_new_line_markers: bool,
    remove_trailing_whitespace: bool,
    remove_trailing_empty_lines: bool,
    new_line_marker: NewLineMarkerMode,
    normalize_empty_files: EmptyFileReplacementMode,
    normalize_whitespace_only_files: EmptyFileReplacementMode,
    replace_tabs_with_spaces: isize,
    normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode,
}

enum ChangeType {
    NewLineMarkerAddedToEndOfFile,
    NewLineMarkerRemovedFromEndOfFile,
    ReplacedNewLineMarker,
    RemovedTrailingWhitespace,
    RemovedEmptyLines,
    ReplacedEmptyFileWithOneLine,
    ReplacedWhiteSpaceOnlyFileWithEmptyFile,
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
            Self::ReplacedWhiteSpaceOnlyFileWithEmptyFile => "ReplacedWhiteSpaceOnlyFileWithEmptyFile",
            Self::ReplacedTabWithSpaces => "ReplacedTabWithSpaces",
            Self::RemovedTab => "RemovedTab",
            Self::ReplacedNonstandardWhitespaceWithSpace => "ReplacedNonstandardWhitespaceWithSpace",
            Self::RemovedNonstandardWhitespace => "RemovedNonstandardWhitespace",
        };
        write!(f, "{:?}", printable)
    }
}

struct Change {
    line_number: usize,
    change_type: ChangeType,
}

fn push_new_line_marker(output: &mut Vec<u8>, output_new_line_marker: &NewLineMarker) {
    match output_new_line_marker {
        NewLineMarker::Linux => {
            output.push(LINE_FEED);
        },
        NewLineMarker::MacOs => {
            output.push(CARRIAGE_RETURN);
        },
        NewLineMarker::Windows => {
            output.push(CARRIAGE_RETURN);
            output.push(LINE_FEED);
        },
    }
}

fn process_file(input: &[u8], options: &Options) -> (Vec<u8>, Vec<Change>) {
    // Figure out what new line marker to write to the output buffer.
    let output_new_line_marker = match options.new_line_marker {
        NewLineMarkerMode::Auto => find_most_common_new_line_marker(input),
        NewLineMarkerMode::Linux => NewLineMarker::Linux,
        NewLineMarkerMode::MacOs => NewLineMarker::MacOs,
        NewLineMarkerMode::Windows => NewLineMarker::Windows,
    };

    // Index into the input buffer.
    let mut i: usize = 0;

    // The output buffer.
    // The output is expected to be about the same size as the input.
    // If there are no changes, the output will be exactly the same as input.
    // In order to avoid too many reallocations, reserve the capacity
    // equal to the size of the input buffer.
    let mut output: Vec<u8> = Vec::with_capacity(input.len());

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

    while i < input.len() {
        if input[i] == CARRIAGE_RETURN || input[i] == LINE_FEED {

            // Parse the new line marker
            let mut new_line_marker: NewLineMarker;
            if input[i] == LINE_FEED {
                new_line_marker = NewLineMarker::Linux;
            } else if i < input.len() - 1 && input[i + 1] == LINE_FEED {
                new_line_marker = NewLineMarker::Windows;
                // Windows new line marker consists of two bytes.
                // Skip the extra byte.
                i += 1;
            } else {
                new_line_marker = NewLineMarker::MacOs;
            }

            // Remove trailing whitespace
            if options.remove_trailing_whitespace && last_non_whitespace < output.len() {
                changes.push(Change{line_number: line_number, change_type: ChangeType::RemovedTrailingWhitespace});
                output.truncate(last_non_whitespace);
            }

            // Determine if the last line is empty
            let is_empty_line: bool = last_end_of_line_including_eol_marker == output.len();

            // Add new line marker
            last_end_of_line_excluding_eol_marker = output.len();
            if options.normalize_new_line_markers && output_new_line_marker != new_line_marker {
                changes.push(Change { line_number: line_number, change_type: ChangeType::ReplacedNewLineMarker });
                push_new_line_marker(&mut output, &output_new_line_marker);
            } else {
                push_new_line_marker(&mut output, &new_line_marker);
            }
            last_end_of_line_including_eol_marker = output.len();

            // Update position of last non-empty line.
            if !is_empty_line {
                last_end_of_non_empty_line_excluding_eol_marker = last_end_of_line_excluding_eol_marker;
                last_end_of_non_empty_line_including_eol_marker = last_end_of_line_including_eol_marker;
                last_non_empty_line_number = line_number;
            }
            line_number += 1;
        } else if input[i] == SPACE {
            output.push(input[i]);
        } else if input[i] == TAB {
            if options.replace_tabs_with_spaces < 0 {
                output.push(input[i]);
            } else if options.replace_tabs_with_spaces > 0 {
                changes.push(Change{ line_number: line_number, change_type: ChangeType::ReplacedTabWithSpaces });
                for _ in 0..options.replace_tabs_with_spaces {
                    output.push(SPACE);
                }
            } else {
                // Remove the tab character.
                changes.push(Change{ line_number: line_number, change_type: ChangeType::RemovedTab });
            }
        } else if input[i] == VERTICAL_TAB || input[i] == FORM_FEED {
            match options.normalize_non_standard_whitespace {
                NonStandardWhitespaceReplacementMode::Ignore => {
                    output.push(input[i]);
                },
                NonStandardWhitespaceReplacementMode::ReplaceWithSpace => {
                    output.push(SPACE);
                    changes.push(Change{line_number: line_number, change_type: ChangeType::ReplacedNonstandardWhitespaceWithSpace});
                },
                NonStandardWhitespaceReplacementMode::Remove => {
                    // Remove the non-standard whitespace character.
                    changes.push(Change{line_number: line_number, change_type: ChangeType::RemovedNonstandardWhitespace});
                },
            }
        } else {
            output.push(input[i]);
            last_non_whitespace = output.len();
        }

        // Move to the next byte
        i += 1;
    }

    // Remove trailing whitespace from the last line.
    if options.remove_trailing_whitespace && last_end_of_line_including_eol_marker < output.len() && last_non_whitespace < output.len() {
        changes.push(Change{line_number: line_number, change_type: ChangeType::RemovedTrailingWhitespace});
        output.truncate(last_non_whitespace);
    }

    // Remove trailing empty lines.
    if options.remove_trailing_empty_lines && last_end_of_line_including_eol_marker == output.len() && last_end_of_non_empty_line_including_eol_marker < output.len() {
        line_number = last_non_empty_line_number + 1;
        last_end_of_line_excluding_eol_marker = last_end_of_non_empty_line_excluding_eol_marker;
        last_end_of_line_including_eol_marker = last_end_of_non_empty_line_including_eol_marker;
        changes.push(Change{ line_number: line_number, change_type: ChangeType::RemovedEmptyLines});
        output.truncate(last_end_of_non_empty_line_including_eol_marker);
    }

    // Add new line marker at the end of the file
    if options.add_new_line_marker_at_end_of_file && last_end_of_line_including_eol_marker < output.len() {
        last_end_of_line_excluding_eol_marker = output.len();
        changes.push(Change{ line_number: line_number, change_type: ChangeType::NewLineMarkerAddedToEndOfFile});
        push_new_line_marker(&mut output, &output_new_line_marker);
        last_end_of_line_including_eol_marker = output.len();
        line_number += 1;
    }

    // Remove new line marker from the end of the file
    if options.remove_new_line_marker_from_end_of_file && last_end_of_line_including_eol_marker == output.len() && line_number >= 2 {
        line_number -= 1;
        changes.push(Change{ line_number: line_number, change_type: ChangeType::NewLineMarkerRemovedFromEndOfFile});
        output.truncate(last_end_of_line_excluding_eol_marker);
    }

    return (output, changes);
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

fn main() {
    // Print hello world.
    println!("Hello, world!");

    // Print command line arguments.
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    // Print content of a file.
    let data: Vec<u8> = fs::read(&FILE_NAME).unwrap();
    dbg!(String::from_utf8_lossy(&data));

    let new_line_marker: NewLineMarker = find_most_common_new_line_marker(&data);
    println!("Most common new line marker is {}", new_line_marker);

    let options: Options = Options {
        add_new_line_marker_at_end_of_file: true,
        remove_new_line_marker_from_end_of_file: false,
        normalize_new_line_markers: true,
        remove_trailing_whitespace: true,
        remove_trailing_empty_lines: true,
        new_line_marker: NewLineMarkerMode::Linux,
        normalize_empty_files: EmptyFileReplacementMode::Empty,
        normalize_whitespace_only_files: EmptyFileReplacementMode::Empty,
        replace_tabs_with_spaces: -1,
        normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode::Ignore,
    };
    let (output, changes): (Vec<u8>, Vec<Change>) = process_file(&data, &options);

    println!("Number of changes {}", changes.len());
    for change in changes {
        println!("Line {}: {}", change.line_number, change.change_type);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_most_common_new_line_marker() {
        assert_eq!(find_most_common_new_line_marker(&[]), NewLineMarker::Linux);
        assert_eq!(find_most_common_new_line_marker(&[LINE_FEED]), NewLineMarker::Linux);
        assert_eq!(find_most_common_new_line_marker(&[CARRIAGE_RETURN]), NewLineMarker::MacOs);
        assert_eq!(find_most_common_new_line_marker(&[CARRIAGE_RETURN, LINE_FEED]), NewLineMarker::Windows);
        assert_eq!(find_most_common_new_line_marker(b"hello world"), NewLineMarker::Linux);
        assert_eq!(find_most_common_new_line_marker(b"a\rb\nc\n"), NewLineMarker::Linux);
        assert_eq!(find_most_common_new_line_marker(b"a\rb\rc\r\n"), NewLineMarker::MacOs);
        assert_eq!(find_most_common_new_line_marker(b"a\r\nb\r\nc\n"), NewLineMarker::Windows);
    }

    #[test]
    fn test_process_file() {
        let options: Options = Options {
            add_new_line_marker_at_end_of_file: true,
            remove_new_line_marker_from_end_of_file: false,
            normalize_new_line_markers: true,
            remove_trailing_whitespace: true,
            remove_trailing_empty_lines: true,
            new_line_marker: NewLineMarkerMode::Linux,
            normalize_empty_files: EmptyFileReplacementMode::Empty,
            normalize_whitespace_only_files: EmptyFileReplacementMode::Empty,
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
            new_line_marker: NewLineMarkerMode::Linux,
            normalize_empty_files: EmptyFileReplacementMode::Empty,
            normalize_whitespace_only_files: EmptyFileReplacementMode::Empty,
            replace_tabs_with_spaces: -1,
            normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode::Ignore,
        };
        assert_eq!(process_file(b"hello world\r\n", &options).0, b"hello world\r\n");
    }

}
