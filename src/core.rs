// Library imports
use std::cmp::max;
use std::fmt;
use std::fs;
use std::path::PathBuf;

// Internal imports
use crate::change::Change;
use crate::change::ChangeType;
use crate::cli::CommandLineArguments;
use crate::cli::NonStandardWhitespaceReplacementMode;
use crate::cli::OutputNewLineMarkerMode;
use crate::cli::TrivialFileReplacementMode;
use crate::error::die;
use crate::error::Error;
use crate::writer::CountingWriter;
use crate::writer::Writer;

// ASCII codes of characters that we care about.
// For efficiency, we encode the characters as unsigned bytes.
// This way we avoid Unicode character decoding and encoding.
const CARRIAGE_RETURN: u8 = b'\r';
const LINE_FEED: u8 = b'\n';
const SPACE: u8 = b' ';
const TAB: u8 = b'\t';
const VERTICAL_TAB: u8 = 0x0B; // The same as '\v' in C, C++, Java and Python.
const FORM_FEED: u8 = 0x0C; // The same as '\f' in C, C++, Java and Python.

/// Converts an ASCII code to a human-readable string.
pub fn char_to_str(char: u8) -> &'static str {
    match char {
        CARRIAGE_RETURN => "\\n",
        LINE_FEED => "\\r",
        SPACE => " ",
        TAB => "\\t",
        VERTICAL_TAB => "\\v",
        FORM_FEED => "\\f",
        _ => "?",
    }
}

/// Type of new line marker. There are three types new line markers:
/// 1) Linux (`\n`)
/// 2) MacOS (`\r`)
/// 3) Windows/DOS (`\r\n`)
#[derive(PartialEq, Debug, Clone)]
pub enum NewLineMarker {
    // Linux line ending is a single line feed character '\n'.
    Linux,

    // MacOS line ending is a single carriage return character '\r'.
    MacOs,

    // Windows/DOS line ending is a sequence of two characters:
    // carriage return character followed by line feed character.
    Windows,
}

impl NewLineMarker {
    /// Byte representation of a new line marker.
    ///
    fn to_bytes(&self) -> &'static [u8] {
        match &self {
            NewLineMarker::Linux => &[LINE_FEED],
            NewLineMarker::MacOs => &[CARRIAGE_RETURN],
            NewLineMarker::Windows => &[CARRIAGE_RETURN, LINE_FEED],
        }
    }
}

impl fmt::Display for NewLineMarker {
    /// Human-readable visible non-whitespace representation
    /// of a new line marker. This function is used in user reporting.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            NewLineMarker::Linux => f.write_str("\\n"),
            NewLineMarker::MacOs => f.write_str("\\r"),
            NewLineMarker::Windows => f.write_str("\\r\\n"),
        }
    }
}

/// Options for formatting a single file.
pub struct Options {
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

impl CommandLineArguments {
    /// Extracts formatting options from command line arguments.
    pub fn get_options(&self) -> Options {
        Options {
            add_new_line_marker_at_end_of_file: self.add_new_line_marker_at_end_of_file,
            remove_new_line_marker_from_end_of_file: self.remove_new_line_marker_from_end_of_file,
            normalize_new_line_markers: self.normalize_new_line_markers,
            remove_trailing_whitespace: self.remove_trailing_whitespace,
            remove_trailing_empty_lines: self.remove_trailing_empty_lines,
            new_line_marker: self.new_line_marker.clone(),
            normalize_empty_files: self.normalize_empty_files.clone(),
            normalize_whitespace_only_files: self.normalize_whitespace_only_files.clone(),
            replace_tabs_with_spaces: self.replace_tabs_with_spaces,
            normalize_non_standard_whitespace: self.normalize_non_standard_whitespace.clone(),
        }
    }
}

/// Determines if a file consists of only whitespace.
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
    true
}

/// Computes the most common new line marker based on content of the file.
/// If there are ties, prefer Linux to Windows to MacOS.
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

    if macos_count > windows_count && macos_count > linux_count {
        return NewLineMarker::MacOs;
    } else if windows_count > linux_count {
        return NewLineMarker::Windows;
    }
    NewLineMarker::Linux
}

/// The core formatting algorithm for making changes in a file.
/// The output is written using a writer. A writer is an in-memory buffer
/// that supports writing bytes and rewinds. The rewinds are used when deleting
/// trailing whitespace.
fn modify_content<T: Writer>(input_data: &[u8], options: &Options, writer: &mut T) -> Vec<Change> {
    // Figure out what new line marker to use when writing to the output buffer.
    let output_new_line_marker = match options.new_line_marker {
        OutputNewLineMarkerMode::Auto => find_most_common_new_line_marker(input_data),
        OutputNewLineMarkerMode::Linux => NewLineMarker::Linux,
        OutputNewLineMarkerMode::MacOs => NewLineMarker::MacOs,
        OutputNewLineMarkerMode::Windows => NewLineMarker::Windows,
    };

    // Handle empty file.
    if input_data.is_empty() {
        return match options.normalize_empty_files {
            TrivialFileReplacementMode::Empty | TrivialFileReplacementMode::Ignore => Vec::new(),
            TrivialFileReplacementMode::OneLine => {
                writer.write_bytes(output_new_line_marker.to_bytes());
                Vec::from([Change::new(1, ChangeType::ReplacedEmptyFileWithOneLine)])
            }
        };
    }

    // Handle non-empty file consisting of whitespace only.
    if is_file_whitespace(input_data) {
        return match options.normalize_whitespace_only_files {
            TrivialFileReplacementMode::Empty => Vec::from([Change::new(
                1,
                ChangeType::ReplacedWhiteSpaceOnlyFileWithEmptyFile,
            )]),
            TrivialFileReplacementMode::Ignore => {
                writer.write_bytes(input_data);
                Vec::new()
            }
            TrivialFileReplacementMode::OneLine => {
                writer.write_bytes(output_new_line_marker.to_bytes());
                if input_data == output_new_line_marker.to_bytes() {
                    Vec::new()
                } else {
                    Vec::from([Change::new(
                        1,
                        ChangeType::ReplacedWhiteSpaceOnlyFileWithOneLine,
                    )])
                }
            }
        };
    }

    // Index into the input buffer.
    let mut i: usize = 0;

    // List of changes between input and output.
    let mut changes: Vec<Change> = Vec::new();

    // Line number. It is incremented every time we encounter a new end of line marker.
    let mut line_number: usize = 1;

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
            if options.remove_trailing_whitespace
                && max(last_non_whitespace, last_end_of_line_including_eol_marker)
                    < writer.position()
            {
                changes.push(Change::new(
                    line_number,
                    ChangeType::RemovedTrailingWhitespace,
                ));
                writer.rewind(max(
                    last_non_whitespace,
                    last_end_of_line_including_eol_marker,
                ));
            }

            // Determine if the last line is empty
            let is_empty_line: bool = last_end_of_line_including_eol_marker == writer.position();

            // Position one byte past the end of last line in the output buffer
            // excluding the last end of line marker.
            let last_end_of_line_excluding_eol_marker: usize = writer.position();

            // Add new line marker
            if options.normalize_new_line_markers && output_new_line_marker != new_line_marker {
                changes.push(Change::new(
                    line_number,
                    ChangeType::ReplacedNewLineMarker(
                        new_line_marker.clone(),
                        output_new_line_marker.clone(),
                    ),
                ));
                writer.write_bytes(output_new_line_marker.to_bytes());
            } else {
                writer.write_bytes(new_line_marker.to_bytes());
            }
            last_end_of_line_including_eol_marker = writer.position();

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
            writer.write(input_data[i]);
        } else if input_data[i] == TAB {
            if options.replace_tabs_with_spaces < 0 {
                writer.write(input_data[i]);
            } else if options.replace_tabs_with_spaces > 0 {
                changes.push(Change::new(line_number, ChangeType::ReplacedTabWithSpaces));
                for _ in 0..options.replace_tabs_with_spaces {
                    writer.write(SPACE);
                }
            } else {
                // Remove the tab character.
                changes.push(Change::new(line_number, ChangeType::RemovedTab));
            }
        } else if input_data[i] == VERTICAL_TAB || input_data[i] == FORM_FEED {
            match options.normalize_non_standard_whitespace {
                NonStandardWhitespaceReplacementMode::Ignore => {
                    writer.write(input_data[i]);
                }
                NonStandardWhitespaceReplacementMode::ReplaceWithSpace => {
                    writer.write(SPACE);
                    changes.push(Change::new(
                        line_number,
                        ChangeType::ReplacedNonstandardWhitespaceBySpace(input_data[i]),
                    ));
                }
                NonStandardWhitespaceReplacementMode::Remove => {
                    // Remove the non-standard whitespace character.
                    changes.push(Change::new(
                        line_number,
                        ChangeType::RemovedNonstandardWhitespace(input_data[i]),
                    ));
                }
            }
        } else {
            writer.write(input_data[i]);
            last_non_whitespace = writer.position();
        }

        // Move to the next byte
        i += 1;
    }

    // Remove trailing whitespace from the last line.
    if options.remove_trailing_whitespace
        && last_end_of_line_including_eol_marker < writer.position()
        && last_non_whitespace < writer.position()
    {
        changes.push(Change::new(
            line_number,
            ChangeType::RemovedTrailingWhitespace,
        ));
        writer.rewind(last_non_whitespace);
    }

    // Remove trailing empty lines.
    if options.remove_trailing_empty_lines
        && last_end_of_line_including_eol_marker == writer.position()
        && last_end_of_non_empty_line_including_eol_marker < writer.position()
    {
        line_number = last_non_empty_line_number + 1;
        last_end_of_line_including_eol_marker = last_end_of_non_empty_line_including_eol_marker;
        changes.push(Change::new(line_number, ChangeType::RemovedEmptyLines));
        writer.rewind(last_end_of_non_empty_line_including_eol_marker);
    }

    // Add new line marker at the end of the file
    if options.add_new_line_marker_at_end_of_file
        && last_end_of_line_including_eol_marker < writer.position()
    {
        changes.push(Change::new(
            line_number,
            ChangeType::NewLineMarkerAddedToEndOfFile,
        ));
        writer.write_bytes(output_new_line_marker.to_bytes());
        last_end_of_line_including_eol_marker = writer.position();
        line_number += 1;
    }

    // Remove new line marker from the end of the file
    if options.remove_new_line_marker_from_end_of_file
        && last_end_of_line_including_eol_marker == writer.position()
        && line_number >= 2
    {
        line_number = last_non_empty_line_number;
        changes.push(Change::new(
            line_number,
            ChangeType::NewLineMarkerRemovedFromEndOfFile,
        ));
        writer.rewind(last_end_of_non_empty_line_excluding_eol_marker);
    }

    changes
}

/// Formats or checks a single file and returns the list of changes tha have been
/// made or would have been made. If check_only is set to true, the file is not modified.
/// Otherwise, the file is overwritten in place.
pub fn process_file(file_path: &PathBuf, options: &Options, check_only: bool) -> Vec<Change> {
    match fs::read(file_path) {
        Err(_) => {
            die(Error::CannotReadFile(file_path.display().to_string()));
        }
        Ok(input_data) => {
            let mut counting_writer = CountingWriter::new();
            let changes: Vec<Change> = modify_content(&input_data, options, &mut counting_writer);
            if !check_only && !changes.is_empty() {
                let mut output_writer = Vec::with_capacity(counting_writer.maximum_position());
                modify_content(&input_data, options, &mut output_writer);
                if fs::write(file_path, output_writer).is_err() {
                    die(Error::CannotWriteFile(file_path.display().to_string()));
                };
            }
            changes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::discover_files;

    impl Options {
        fn new() -> Self {
            Self {
                add_new_line_marker_at_end_of_file: false,
                remove_new_line_marker_from_end_of_file: false,
                normalize_new_line_markers: false,
                remove_trailing_whitespace: false,
                remove_trailing_empty_lines: false,
                new_line_marker: OutputNewLineMarkerMode::Auto,
                normalize_empty_files: TrivialFileReplacementMode::Ignore,
                normalize_whitespace_only_files: TrivialFileReplacementMode::Ignore,
                replace_tabs_with_spaces: -1,
                normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode::Ignore,
            }
        }

        fn add_new_line_marker_at_end_of_file(mut self) -> Self {
            self.add_new_line_marker_at_end_of_file = true;
            self.remove_new_line_marker_from_end_of_file = false;
            return self;
        }

        fn remove_new_line_marker_from_end_of_file(mut self) -> Self {
            self.remove_new_line_marker_from_end_of_file = true;
            self.add_new_line_marker_at_end_of_file = false;
            return self;
        }

        fn normalize_new_line_markers(mut self) -> Self {
            self.normalize_new_line_markers = true;
            return self;
        }

        fn remove_trailing_whitespace(mut self) -> Self {
            self.remove_trailing_whitespace = true;
            return self;
        }

        fn remove_trailing_empty_lines(mut self) -> Self {
            self.remove_trailing_empty_lines = true;
            return self;
        }

        fn new_line_marker(mut self, output_new_line_marker_mode: OutputNewLineMarkerMode) -> Self {
            self.new_line_marker = output_new_line_marker_mode;
            return self;
        }

        fn normalize_empty_files(mut self, mode: TrivialFileReplacementMode) -> Self {
            self.normalize_empty_files = mode;
            return self;
        }

        fn normalize_whitespace_only_files(mut self, mode: TrivialFileReplacementMode) -> Self {
            self.normalize_whitespace_only_files = mode;
            return self;
        }

        fn replace_tabs_with_spaces(mut self, num_spaces: isize) -> Self {
            self.replace_tabs_with_spaces = num_spaces;
            return self;
        }

        fn normalize_non_standard_whitespace(
            mut self,
            mode: NonStandardWhitespaceReplacementMode,
        ) -> Self {
            self.normalize_non_standard_whitespace = mode;
            return self;
        }
    }

    #[test]
    fn test_is_file_whitespace() {
        assert_eq!(is_file_whitespace(&[]), true);
        assert_eq!(is_file_whitespace(b"    "), true);
        assert_eq!(is_file_whitespace(b"\n\n\n"), true);
        assert_eq!(is_file_whitespace(b"\r\r\r"), true);
        assert_eq!(is_file_whitespace(b" \t\n\r"), true);
        assert_eq!(is_file_whitespace(b"hello"), false);
        assert_eq!(is_file_whitespace(b"hello world\n"), false);
        assert_eq!(is_file_whitespace(b"\n\t \x0B \x0C \n  "), true);
    }

    #[test]
    fn test_find_most_common_new_line_marker() {
        assert_eq!(find_most_common_new_line_marker(&[]), NewLineMarker::Linux);
        assert_eq!(
            find_most_common_new_line_marker(b"\n"),
            NewLineMarker::Linux
        );
        assert_eq!(
            find_most_common_new_line_marker(b"\r"),
            NewLineMarker::MacOs
        );
        assert_eq!(
            find_most_common_new_line_marker(b"\r\n"),
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
        assert_eq!(
            find_most_common_new_line_marker(b"\n\n\r\r\r\n\r\n"),
            NewLineMarker::Linux,
        );
        assert_eq!(
            find_most_common_new_line_marker(b"\n\r\r\r\n\r\n"),
            NewLineMarker::Windows,
        );
        assert_eq!(
            find_most_common_new_line_marker(b"\n\r\r\r\n"),
            NewLineMarker::MacOs,
        );
    }

    #[test]
    fn test_modify_content_do_nothing() {
        let options: Options = Options::new();
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  ");
        assert!(changes.is_empty());
    }

    #[test]
    fn test_modify_content_do_nothing_whitespace_only_file() {
        let options: Options = Options::new();
        let mut output = Vec::new();
        let changes = modify_content(b"  ", &options, &mut output);
        assert_eq!(output, b"  ");
        assert!(changes.is_empty());
    }

    #[test]
    fn test_modify_content_do_nothing_empty_file() {
        let options: Options = Options::new();
        let mut output = Vec::new();
        let changes = modify_content(b"", &options, &mut output);
        assert_eq!(output, b"");
        assert!(changes.is_empty());
    }

    #[test]
    fn test_modify_content_add_new_line_marker_auto() {
        let options: Options = Options::new().add_new_line_marker_at_end_of_file();
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  \r\n");
        assert_eq!(
            changes,
            vec![Change::new(3, ChangeType::NewLineMarkerAddedToEndOfFile)]
        );
    }

    #[test]
    fn test_modify_content_add_new_line_marker_linux() {
        let options: Options = Options::new()
            .add_new_line_marker_at_end_of_file()
            .new_line_marker(OutputNewLineMarkerMode::Linux);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  \n");
        assert_eq!(
            changes,
            vec![Change::new(3, ChangeType::NewLineMarkerAddedToEndOfFile)]
        );
    }

    #[test]
    fn test_modify_content_add_new_line_marker_macos() {
        let options: Options = Options::new()
            .add_new_line_marker_at_end_of_file()
            .new_line_marker(OutputNewLineMarkerMode::MacOs);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  \r");
        assert_eq!(
            changes,
            vec![Change::new(3, ChangeType::NewLineMarkerAddedToEndOfFile)]
        );
    }

    #[test]
    fn test_modify_content_add_new_line_marker_windows() {
        let options: Options = Options::new()
            .add_new_line_marker_at_end_of_file()
            .new_line_marker(OutputNewLineMarkerMode::Windows);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  \r\n");
        assert_eq!(
            changes,
            vec![Change::new(3, ChangeType::NewLineMarkerAddedToEndOfFile)]
        );
    }

    #[test]
    fn test_modify_content_remove_new_line_marker_from_end_of_file_1() {
        let options: Options = Options::new().remove_new_line_marker_from_end_of_file();
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  \n", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  ");
        assert_eq!(
            changes,
            vec![Change::new(
                3,
                ChangeType::NewLineMarkerRemovedFromEndOfFile
            )]
        );
    }

    #[test]
    fn test_modify_content_remove_new_line_marker_from_end_of_file_2() {
        let options: Options = Options::new().remove_new_line_marker_from_end_of_file();
        let mut output = Vec::new();
        let changes = modify_content(b"hello", &options, &mut output);
        assert_eq!(output, b"hello");
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn test_modify_content_remove_new_line_marker_from_end_of_file_3() {
        let options: Options = Options::new().remove_new_line_marker_from_end_of_file();
        let mut output = Vec::new();
        let changes = modify_content(b"", &options, &mut output);
        assert_eq!(output, b"");
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn test_modify_content_remove_new_line_marker_from_end_of_file_4() {
        let options: Options = Options::new().remove_new_line_marker_from_end_of_file();
        let mut output = Vec::new();
        let changes = modify_content(b"hello  \n\r\n\r", &options, &mut output);
        assert_eq!(output, b"hello  ");
        assert_eq!(
            changes,
            vec![Change::new(
                1,
                ChangeType::NewLineMarkerRemovedFromEndOfFile
            ),]
        );
    }

    #[test]
    fn test_modify_content_normalize_new_line_markers_auto() {
        let options: Options = Options::new().normalize_new_line_markers();
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  \r\n", &options, &mut output);
        assert_eq!(output, b"hello\r\n\r\nworld  \r\n");
        assert_eq!(
            changes,
            vec![Change::new(
                2,
                ChangeType::ReplacedNewLineMarker(NewLineMarker::MacOs, NewLineMarker::Windows)
            )]
        );
    }

    #[test]
    fn test_modify_content_normalize_new_line_markers_linux() {
        let options: Options = Options::new()
            .normalize_new_line_markers()
            .new_line_marker(OutputNewLineMarkerMode::Linux);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  \r\n", &options, &mut output);
        assert_eq!(output, b"hello\n\nworld  \n");
        assert_eq!(
            changes,
            vec![
                Change::new(
                    1,
                    ChangeType::ReplacedNewLineMarker(NewLineMarker::Windows, NewLineMarker::Linux)
                ),
                Change::new(
                    2,
                    ChangeType::ReplacedNewLineMarker(NewLineMarker::MacOs, NewLineMarker::Linux)
                ),
                Change::new(
                    3,
                    ChangeType::ReplacedNewLineMarker(NewLineMarker::Windows, NewLineMarker::Linux)
                ),
            ]
        );
    }

    #[test]
    fn test_modify_content_normalize_new_line_markers_macos() {
        let options: Options = Options::new()
            .normalize_new_line_markers()
            .new_line_marker(OutputNewLineMarkerMode::MacOs);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  \r\n", &options, &mut output);
        assert_eq!(output, b"hello\r\rworld  \r");
        assert_eq!(
            changes,
            vec![
                Change::new(
                    1,
                    ChangeType::ReplacedNewLineMarker(NewLineMarker::Windows, NewLineMarker::MacOs)
                ),
                Change::new(
                    3,
                    ChangeType::ReplacedNewLineMarker(NewLineMarker::Windows, NewLineMarker::MacOs)
                ),
            ]
        );
    }

    #[test]
    fn test_modify_content_normalize_new_line_markers_windows() {
        let options: Options = Options::new()
            .normalize_new_line_markers()
            .new_line_marker(OutputNewLineMarkerMode::Windows);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  \r\n", &options, &mut output);
        assert_eq!(output, b"hello\r\n\r\nworld  \r\n");
        assert_eq!(
            changes,
            vec![Change::new(
                2,
                ChangeType::ReplacedNewLineMarker(NewLineMarker::MacOs, NewLineMarker::Windows)
            )]
        );
    }

    #[test]
    fn test_modify_content_remove_trailing_empty_lines() {
        let options: Options = Options::new().remove_trailing_empty_lines();
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld\r\n\n\n\n\n\n", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld\r\n");
        assert_eq!(changes, vec![Change::new(4, ChangeType::RemovedEmptyLines)]);
    }

    #[test]
    fn test_modify_content_remove_trailing_whitespace_1() {
        let options: Options = Options::new().remove_trailing_whitespace();
        let mut output = Vec::new();
        let changes = modify_content(b"hello world   ", &options, &mut output);
        assert_eq!(output, b"hello world");
        assert_eq!(
            changes,
            vec![Change::new(1, ChangeType::RemovedTrailingWhitespace)]
        );
    }

    #[test]
    fn test_modify_content_remove_trailing_whitespace_2() {
        let options: Options = Options::new().remove_trailing_whitespace();
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld   ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld");
        assert_eq!(
            changes,
            vec![Change::new(3, ChangeType::RemovedTrailingWhitespace)]
        );
    }

    #[test]
    fn test_modify_content_remove_trailing_whitespace_3() {
        let options: Options = Options::new().remove_trailing_whitespace();
        let mut output = Vec::new();
        let changes = modify_content(b"hello \t  \r\n \t  \rworld   ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld");
        assert_eq!(
            changes,
            vec![
                Change::new(1, ChangeType::RemovedTrailingWhitespace),
                Change::new(2, ChangeType::RemovedTrailingWhitespace),
                Change::new(3, ChangeType::RemovedTrailingWhitespace)
            ]
        );
    }

    #[test]
    fn test_modify_content_remove_trailing_whitespace_4() {
        let options: Options = Options::new().remove_trailing_whitespace();
        let mut output = Vec::new();
        let changes = modify_content(b"hello world   \n\n   \n", &options, &mut output);
        assert_eq!(output, b"hello world\n\n\n");
        assert_eq!(
            changes,
            vec![
                Change::new(1, ChangeType::RemovedTrailingWhitespace),
                Change::new(3, ChangeType::RemovedTrailingWhitespace),
            ]
        );
    }

    #[test]
    fn test_modify_content_remove_trailing_whitespace_5() {
        let options: Options = Options::new().remove_trailing_whitespace();
        let mut output = Vec::new();
        let changes = modify_content(b"hello world   \x0C  \n\n \x0B \n", &options, &mut output);
        assert_eq!(output, b"hello world\n\n\n");
        assert_eq!(
            changes,
            vec![
                Change::new(1, ChangeType::RemovedTrailingWhitespace),
                Change::new(3, ChangeType::RemovedTrailingWhitespace),
            ]
        );
    }

    #[test]
    fn test_modify_content_remove_trailing_whitespace_and_normalize_non_standard_whitespace_1() {
        let options: Options = Options::new()
            .remove_trailing_whitespace()
            .normalize_non_standard_whitespace(NonStandardWhitespaceReplacementMode::Remove);
        let mut output = Vec::new();
        let changes = modify_content(b"hello world   \x0C  \n\n \x0B \n", &options, &mut output);
        assert_eq!(output, b"hello world\n\n\n");
        assert_eq!(
            changes,
            vec![
                Change::new(1, ChangeType::RemovedNonstandardWhitespace(0x0C)),
                Change::new(1, ChangeType::RemovedTrailingWhitespace),
                Change::new(3, ChangeType::RemovedNonstandardWhitespace(0x0B)),
                Change::new(3, ChangeType::RemovedTrailingWhitespace),
            ]
        );
    }

    #[test]
    fn test_modify_content_remove_trailing_whitespace_and_normalize_non_standard_whitespace_2() {
        let options: Options = Options::new()
            .remove_trailing_whitespace()
            .normalize_non_standard_whitespace(
                NonStandardWhitespaceReplacementMode::ReplaceWithSpace,
            );
        let mut output = Vec::new();
        let changes = modify_content(b"hello world   \x0C  \n\n \x0B \n", &options, &mut output);
        assert_eq!(output, b"hello world\n\n\n");
        assert_eq!(
            changes,
            vec![
                Change::new(1, ChangeType::ReplacedNonstandardWhitespaceBySpace(0x0C)),
                Change::new(1, ChangeType::RemovedTrailingWhitespace),
                Change::new(3, ChangeType::ReplacedNonstandardWhitespaceBySpace(0x0B)),
                Change::new(3, ChangeType::RemovedTrailingWhitespace),
            ]
        );
    }

    #[test]
    fn test_modify_content_remove_trailing_whitespace_and_modify_content_remove_trailing_empty_lines(
    ) {
        let options: Options = Options::new()
            .remove_trailing_whitespace()
            .remove_trailing_empty_lines();
        let mut output = Vec::new();
        let changes = modify_content(b"hello world   \n\n   \n", &options, &mut output);
        assert_eq!(output, b"hello world\n");
        assert_eq!(
            changes,
            vec![
                Change::new(1, ChangeType::RemovedTrailingWhitespace),
                Change::new(3, ChangeType::RemovedTrailingWhitespace),
                Change::new(2, ChangeType::RemovedEmptyLines),
            ]
        );
    }

    #[test]
    fn test_modify_content_normalize_empty_files_empty() {
        let options: Options =
            Options::new().normalize_empty_files(TrivialFileReplacementMode::Empty);
        let mut output = Vec::new();
        let changes = modify_content(b"", &options, &mut output);
        assert_eq!(output, b"");
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn test_modify_content_normalize_empty_files_ignore() {
        let options: Options =
            Options::new().normalize_empty_files(TrivialFileReplacementMode::Ignore);
        let mut output = Vec::new();
        let changes = modify_content(b"", &options, &mut output);
        assert_eq!(output, b"");
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn test_modify_content_normalize_empty_files_one_line() {
        let options: Options =
            Options::new().normalize_empty_files(TrivialFileReplacementMode::OneLine);
        let mut output = Vec::new();
        let changes = modify_content(b"", &options, &mut output);
        assert_eq!(output, b"\n");
        assert_eq!(
            changes,
            vec![Change::new(1, ChangeType::ReplacedEmptyFileWithOneLine)]
        );
    }

    #[test]
    fn test_modify_content_normalize_whitespace_only_files_empty() {
        let options: Options =
            Options::new().normalize_whitespace_only_files(TrivialFileReplacementMode::Empty);
        let mut output = Vec::new();
        let changes = modify_content(b"\n\t \x0B \x0C \n  ", &options, &mut output);
        assert_eq!(output, b"");
        assert_eq!(
            changes,
            vec![Change::new(
                1,
                ChangeType::ReplacedWhiteSpaceOnlyFileWithEmptyFile
            )]
        );
    }

    #[test]
    fn test_modify_content_normalize_whitespace_only_files_ignore() {
        let options: Options =
            Options::new().normalize_whitespace_only_files(TrivialFileReplacementMode::Ignore);
        let mut output = Vec::new();
        let changes = modify_content(b"\n\t \x0B \x0C \n  ", &options, &mut output);
        assert_eq!(output, b"\n\t \x0B \x0C \n  ");
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn test_modify_content_normalize_whitespace_only_files_one_line() {
        let options: Options =
            Options::new().normalize_whitespace_only_files(TrivialFileReplacementMode::OneLine);
        let mut output = Vec::new();
        let changes = modify_content(b"\n\t \x0B \x0C \n  ", &options, &mut output);
        assert_eq!(output, b"\n");
        assert_eq!(
            changes,
            vec![Change::new(
                1,
                ChangeType::ReplacedWhiteSpaceOnlyFileWithOneLine
            )]
        );
    }

    #[test]
    fn test_modify_content_normalize_whitespace_only_files_one_line_2() {
        let options: Options =
            Options::new().normalize_whitespace_only_files(TrivialFileReplacementMode::OneLine);
        let mut output = Vec::new();
        let changes = modify_content(b"\n", &options, &mut output);
        assert_eq!(output, b"\n");
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn test_modify_content_normalize_whitespace_only_files_one_line_3() {
        let options: Options = Options::new()
            .normalize_whitespace_only_files(TrivialFileReplacementMode::OneLine)
            .new_line_marker(OutputNewLineMarkerMode::Linux);
        let mut output = Vec::new();
        let changes = modify_content(b"\r\n", &options, &mut output);
        assert_eq!(output, b"\n");
        assert_eq!(
            changes,
            vec![Change::new(
                1,
                ChangeType::ReplacedWhiteSpaceOnlyFileWithOneLine
            )]
        );
    }

    #[test]
    fn test_modify_content_normalize_whitespace_only_files_one_line_4() {
        let options: Options =
            Options::new().normalize_whitespace_only_files(TrivialFileReplacementMode::OneLine);
        let mut output = Vec::new();
        let changes = modify_content(b"\r\n", &options, &mut output);
        assert_eq!(output, b"\r\n");
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn test_modify_content_replace_tabs_with_spaces_ignore() {
        let options: Options = Options::new().replace_tabs_with_spaces(-47);
        let mut output = Vec::new();
        let changes = modify_content(b"\t", &options, &mut output);
        assert_eq!(output, b"\t");
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn test_modify_content_replace_tabs_with_spaces_0() {
        let options: Options = Options::new().replace_tabs_with_spaces(0);
        let mut output = Vec::new();
        let changes = modify_content(b"\thello", &options, &mut output);
        assert_eq!(output, b"hello");
        assert_eq!(changes, vec![Change::new(1, ChangeType::RemovedTab)]);
    }

    #[test]
    fn test_modify_content_replace_tabs_with_spaces_3() {
        let options: Options = Options::new().replace_tabs_with_spaces(3);
        let mut output = Vec::new();
        let changes = modify_content(b"\thello", &options, &mut output);
        assert_eq!(output, b"   hello");
        assert_eq!(
            changes,
            vec![Change::new(1, ChangeType::ReplacedTabWithSpaces)]
        );
    }

    #[test]
    fn test_modify_content_normalize_non_standard_whitespace_ignore() {
        let options: Options = Options::new()
            .normalize_non_standard_whitespace(NonStandardWhitespaceReplacementMode::Ignore);
        let mut output = Vec::new();
        let changes = modify_content(b"\x0B\x0Chello\t ", &options, &mut output);
        assert_eq!(output, b"\x0B\x0Chello\t ");
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn test_modify_content_normalize_non_standard_whitespace_replace_with_space() {
        let options: Options = Options::new().normalize_non_standard_whitespace(
            NonStandardWhitespaceReplacementMode::ReplaceWithSpace,
        );
        let mut output = Vec::new();
        let changes = modify_content(b"\x0B\x0Chello\t ", &options, &mut output);
        assert_eq!(output, b"  hello\t ");
        assert_eq!(
            changes,
            vec![
                Change::new(1, ChangeType::ReplacedNonstandardWhitespaceBySpace(0x0B)),
                Change::new(1, ChangeType::ReplacedNonstandardWhitespaceBySpace(0x0C)),
            ]
        );
    }

    #[test]
    fn test_modify_content_normalize_non_standard_whitespace_remove() {
        let options: Options = Options::new()
            .normalize_non_standard_whitespace(NonStandardWhitespaceReplacementMode::Remove);
        let mut output = Vec::new();
        let changes = modify_content(b"\x0B\x0Chello\t ", &options, &mut output);
        assert_eq!(output, b"hello\t ");
        assert_eq!(
            changes,
            vec![
                Change::new(1, ChangeType::RemovedNonstandardWhitespace(0x0B)),
                Change::new(1, ChangeType::RemovedNonstandardWhitespace(0x0C)),
            ]
        );
    }

    #[test]
    fn test_process_file() {
        let options: Options = Options::new()
            .new_line_marker(OutputNewLineMarkerMode::Linux)
            .add_new_line_marker_at_end_of_file()
            .normalize_new_line_markers()
            .remove_trailing_whitespace()
            .remove_trailing_empty_lines()
            .normalize_non_standard_whitespace(NonStandardWhitespaceReplacementMode::Remove)
            .replace_tabs_with_spaces(4);

        let args = vec![
            "src/",
            ".gitignore",
            "Cargo.lock",
            "Cargo.toml",
            "DEVELOPING.md",
            "LICENSE",
            "README.md",
        ];

        let path_bufs = args.iter().map(|x| PathBuf::from(x)).collect::<Vec<_>>();
        let files = discover_files(&path_bufs, false);

        for file in &files {
            let changes = process_file(file, &options, true);
            assert_eq!(
                changes,
                vec![],
                "The file `{:?}` is not properly formatted.",
                file
            );
        }
    }
}
