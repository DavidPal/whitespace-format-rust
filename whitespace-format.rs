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
use std::cmp;
use std::fmt::{self, Display, Formatter};

const FILE_NAME: &str = "README.md";

// ASCII codes of characters that we care about.
// For efficiency, we encode the characters as unsigned bytes.
// This way we avoid Unicode character decoding and encoding.
const CARRIAGE_RETURN: u8 = b'\r';
const LINE_FEED: u8 = b'\n';
const SPACE: u8 = b' ';
const TAB: u8 = b'\t';
const VERTICAL_TAB: u8 = b'\n'; // The same as '\v' in C, C++, Java and Python.
const FORM_FEED: u8 = 0x0C;  // The same as '\f' in C, C++, Java and Python.

// Possible line ending.
enum NewLineMarker {
    // Linux line ending is a single line feed character '\n'.
    Linux,

    // MacOS line ending is a single carriage return character '\r'.
    MacOs,

    // Windows/DOS line ending is a sequence of two characters:
    // carriage return character followed by line feed character.
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

impl Display for NewLineMarker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    new_line_marker: NewLineMarker,
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
    NormalizedNonstandardWhitespace,
}

impl Display for ChangeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
            Self::NormalizedNonstandardWhitespace => "NormalizedNonstandardWhitespace",
        };
        write!(f, "{:?}", printable)
    }
}

struct Change {
    line_number: usize,
    change_type: ChangeType,
}

fn push_new_line_marker(output: &mut Vec<u8>, new_line_marker: &NewLineMarker) -> usize {
    match new_line_marker {
        NewLineMarker::Linux => {
            output.push(LINE_FEED);
            return 1;
        },
        NewLineMarker::MacOs => {
            output.push(CARRIAGE_RETURN);
            return 1;
        },
        NewLineMarker::Windows => {
            output.push(CARRIAGE_RETURN);
            output.push(LINE_FEED);
            return 2;
        },
    }
}

fn process_file(input: &[u8], options: &Options) -> (Vec<u8>, Vec<Change>) {
    // Index into the input data.
    let mut i: usize = 0;

    // The output buffer.
    // The output is expected to be about the same size as the input.
    // In order to avoid too many reallocations, we reserve
    // the capacity identical to the size of the input.
    let mut output: Vec<u8> = Vec::with_capacity(input.len());

    // List of changes between input and output.
    let mut changes: Vec<Change> = Vec::new();

    // Line number. It is incremented every time we encounter a new end of line marker.
    let mut line_number: usize = 1;

    // Length of the last line including in bytes in the output buffer.
    // The length includes any trailing whitespace.
    // The length excludes the end of line marker.
    let mut last_line_length: usize = 0;

    // Number of trailing whitespace characters on the last line in the output buffer
    // written so far. The number includes spaces, tabs, vertical tab, and form feed characters.
    // It excludes the end of line marker.
    let mut number_trailing_whitespaces_on_line: usize = 0;

    // Number of trailing empty lines written to the output buffer so far.
    let mut number_trailing_empty_lines: usize = 0;

    // Length of last new line marker in the output buffer expressed in bytes.
    //   0 if no end of line marker was written yet.
    //   1 if the last new line marker was a single byte, i.e.,
    //     either a single LINE_FEED character, or a single CARRIAGE_RETURN character.
    //   2 if the last new line marker was a pair of bytes CARRIAGE_RETURN + LINE_FEED.
    let mut last_new_line_marker_length: usize = 0;

    while i < input.len() {
        if input[i] == CARRIAGE_RETURN || input[i] == LINE_FEED {
            // Parse the new line marker.
            let mut new_line_marker: NewLineMarker = NewLineMarker::Linux;
            if input[i] == CARRIAGE_RETURN {
                if i < input.len() - 1 && input[i + 1] == LINE_FEED {
                    new_line_marker = NewLineMarker::Windows;
                    // Windows new line marker consists of two bytes.
                    // Skip the extra byte.
                    i += 1;
                } else {
                    new_line_marker = NewLineMarker::MacOs;
                }
            } else {
                new_line_marker = NewLineMarker::Linux;
            }

            if !matches!(&options.new_line_marker, new_line_marker) {
                changes.push(Change{line_number: line_number, change_type: ChangeType::ReplacedNewLineMarker});
            }
            if options.remove_trailing_whitespace && number_trailing_whitespaces_on_line > 0 {
                last_line_length -= number_trailing_whitespaces_on_line;
                changes.push(Change{line_number: line_number, change_type: ChangeType::RemovedTrailingWhitespace});
                output.truncate(output.len() - number_trailing_whitespaces_on_line);
            }

            // End the current line
            last_new_line_marker_length = push_new_line_marker(&mut output, &options.new_line_marker);

            // Start a new line
            line_number += 1;
            if last_line_length == 0 {
                number_trailing_empty_lines += 1;
            } else {
                number_trailing_empty_lines = 0;
            }
            last_line_length = 0;
            number_trailing_whitespaces_on_line = 0;
        } else if input[i] == SPACE {
            output.push(input[i]);
            last_line_length += 1;
            number_trailing_whitespaces_on_line += 1;
        } else if input[i] == TAB {
            if options.replace_tabs_with_spaces < 0 {
                output.push(input[i]);
                last_line_length += 1;
                number_trailing_whitespaces_on_line += 1;
            } else if options.replace_tabs_with_spaces > 0 {
                last_line_length += options.replace_tabs_with_spaces as usize;
                number_trailing_empty_lines += options.replace_tabs_with_spaces as usize;
                changes.push(Change{ line_number: line_number, change_type: ChangeType::ReplacedTabWithSpaces });
                for _ in 0..options.replace_tabs_with_spaces {
                    output.push(SPACE);
                }
            } else {
                changes.push(Change{ line_number: line_number, change_type: ChangeType::ReplacedTabWithSpaces });
            }
        } else if input[i] == VERTICAL_TAB || input[i] == FORM_FEED {
            match options.normalize_non_standard_whitespace {
                NonStandardWhitespaceReplacementMode::Ignore => {
                    output.push(input[i]);
                    last_line_length += 1;
                    number_trailing_whitespaces_on_line += 1;
                },
                NonStandardWhitespaceReplacementMode::ReplaceWithSpace => {
                    output.push(SPACE);
                    last_line_length += 1;
                    number_trailing_whitespaces_on_line += 1;
                },
                NonStandardWhitespaceReplacementMode::Remove => {
                    // Do nothing
                },
            }
        } else {
            output.push(input[i]);
            last_line_length += 1;
            number_trailing_whitespaces_on_line = 0;
        }

        // Move to the next byte
        i += 1;
    }

    if options.remove_trailing_whitespace && number_trailing_whitespaces_on_line > 0 {
        last_line_length -= number_trailing_whitespaces_on_line;
        changes.push(Change{ line_number: line_number, change_type: ChangeType::RemovedTrailingWhitespace});
        output.truncate(output.len() - number_trailing_whitespaces_on_line);
    }

    if options.add_new_line_marker_at_end_of_file && last_line_length > 0 {
        changes.push(Change{ line_number: line_number, change_type: ChangeType::NewLineMarkerAddedToEndOfFile});
        push_new_line_marker(&mut output, &options.new_line_marker);
    }

    if options.remove_new_line_marker_from_end_of_file && line_number >= 2 && last_line_length == 0 {
        changes.push(Change{ line_number: line_number - 1, change_type: ChangeType::NewLineMarkerRemovedFromEndOfFile});
        output.truncate(output.len() - last_new_line_marker_length);
    }

    return (output, changes);
}

/// Computes the most common new line marker based on content of the file.
/// If there are ties, prefer Linux to MacOS to Windows.
/// If there are no new line markers, return Linux.
fn find_most_common_new_line_marker(data: &[u8]) -> NewLineMarker {
    let mut linux_count: usize = 0;
    let mut macos_count: usize = 0;
    let mut windows_count: usize = 0;
    let mut i: usize = 0;

    while i < data.len() {
        if data[i] == CARRIAGE_RETURN {
            if i < data.len() - 1 && data[i + 1] == LINE_FEED {
                windows_count += 1;
                i += 1;
            } else {
                macos_count += 1;
            }
        } else if data[i] == LINE_FEED {
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
        new_line_marker: NewLineMarker::Linux,
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
