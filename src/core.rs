// Library imports
use std::fmt;
use std::fs;
use std::path::PathBuf;

// Internal imports
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
    pub fn get_options(&self) -> Options {
        Options {
            add_new_line_marker_at_end_of_file: self.add_new_line_marker_at_end_of_file.clone(),
            remove_new_line_marker_from_end_of_file: self
                .remove_new_line_marker_from_end_of_file
                .clone(),
            normalize_new_line_markers: self.normalize_new_line_markers.clone(),
            remove_trailing_whitespace: self.remove_trailing_whitespace.clone(),
            remove_trailing_empty_lines: self.remove_trailing_empty_lines.clone(),
            new_line_marker: self.new_line_marker.clone(),
            normalize_empty_files: self.normalize_empty_files.clone(),
            normalize_whitespace_only_files: self.normalize_whitespace_only_files.clone(),
            replace_tabs_with_spaces: self.replace_tabs_with_spaces.clone(),
            normalize_non_standard_whitespace: self.normalize_non_standard_whitespace.clone(),
        }
    }
}

#[allow(dead_code)]
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
    pub line_number: usize,
    pub change_type: ChangeType,
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
    return true;
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

fn modify_content<T: Writer>(input_data: &[u8], options: &Options, writer: &mut T) -> Vec<Change> {
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
            TrivialFileReplacementMode::Empty | TrivialFileReplacementMode::Ignore => Vec::new(),
            TrivialFileReplacementMode::OneLine => {
                writer.write_bytes(output_new_line_marker.to_bytes());
                Vec::from([Change {
                    line_number: 1,
                    change_type: ChangeType::ReplacedEmptyFileWithOneLine,
                }])
            }
        };
    }

    // Handle non-empty file consisting of whitespace only.
    if is_file_whitespace(input_data) {
        return match options.normalize_whitespace_only_files {
            TrivialFileReplacementMode::Empty => Vec::from([Change {
                line_number: 1,
                change_type: ChangeType::ReplacedWhiteSpaceOnlyFileWithEmptyFile,
            }]),
            TrivialFileReplacementMode::Ignore => {
                writer.write_bytes(input_data);
                Vec::new()
            }
            TrivialFileReplacementMode::OneLine => {
                writer.write_bytes(output_new_line_marker.to_bytes());
                if input_data == output_new_line_marker.to_bytes() {
                    Vec::new()
                } else {
                    Vec::from([Change {
                        line_number: 1,
                        change_type: ChangeType::ReplacedWhiteSpaceOnlyFileWithOneLine,
                    }])
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
            if options.remove_trailing_whitespace && last_non_whitespace < writer.position() {
                changes.push(Change {
                    line_number: line_number,
                    change_type: ChangeType::RemovedTrailingWhitespace,
                });
                writer.rewind(last_non_whitespace);
            }

            // Determine if the last line is empty
            let is_empty_line: bool = last_end_of_line_including_eol_marker == writer.position();

            // Add new line marker
            last_end_of_line_excluding_eol_marker = writer.position();
            if options.normalize_new_line_markers && output_new_line_marker != new_line_marker {
                changes.push(Change {
                    line_number: line_number,
                    change_type: ChangeType::ReplacedNewLineMarker,
                });
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
                changes.push(Change {
                    line_number: line_number,
                    change_type: ChangeType::ReplacedTabWithSpaces,
                });
                for _ in 0..options.replace_tabs_with_spaces {
                    writer.write(SPACE);
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
                    writer.write(input_data[i]);
                }
                NonStandardWhitespaceReplacementMode::ReplaceWithSpace => {
                    writer.write(SPACE);
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
        changes.push(Change {
            line_number: line_number,
            change_type: ChangeType::RemovedTrailingWhitespace,
        });
        writer.rewind(last_non_whitespace);
    }

    // Remove trailing empty lines.
    if options.remove_trailing_empty_lines
        && last_end_of_line_including_eol_marker == writer.position()
        && last_end_of_non_empty_line_including_eol_marker < writer.position()
    {
        line_number = last_non_empty_line_number + 1;
        last_end_of_line_excluding_eol_marker = last_end_of_non_empty_line_excluding_eol_marker;
        last_end_of_line_including_eol_marker = last_end_of_non_empty_line_including_eol_marker;
        changes.push(Change {
            line_number: line_number,
            change_type: ChangeType::RemovedEmptyLines,
        });
        writer.rewind(last_end_of_non_empty_line_including_eol_marker);
    }

    // Add new line marker at the end of the file
    if options.add_new_line_marker_at_end_of_file
        && last_end_of_line_including_eol_marker < writer.position()
    {
        last_end_of_line_excluding_eol_marker = writer.position();
        changes.push(Change {
            line_number: line_number,
            change_type: ChangeType::NewLineMarkerAddedToEndOfFile,
        });
        writer.write_bytes(output_new_line_marker.to_bytes());
        last_end_of_line_including_eol_marker = writer.position();
        line_number += 1;
    }

    // Remove new line marker from the end of the file
    if options.remove_new_line_marker_from_end_of_file
        && last_end_of_line_including_eol_marker == writer.position()
        && line_number >= 2
    {
        line_number -= 1;
        changes.push(Change {
            line_number: line_number,
            change_type: ChangeType::NewLineMarkerRemovedFromEndOfFile,
        });
        writer.rewind(last_end_of_line_excluding_eol_marker);
    }

    return changes;
}

pub fn process_file(file_path: &PathBuf, options: &Options, check_only: bool) -> usize {
    println!("Processing file '{}'...", file_path.display());
    let input_data: Vec<u8> = fs::read(file_path).unwrap_or_else(|_error| {
        die(Error::CannotReadFile(file_path.display().to_string()));
    });

    let mut counting_writer = CountingWriter::new();
    let changes: Vec<Change> = modify_content(&input_data, options, &mut counting_writer);

    if !check_only {
        let mut output_writer = Vec::with_capacity(counting_writer.position());
        modify_content(&input_data, options, &mut output_writer);

        fs::write(file_path, output_writer).unwrap_or_else(|_error| {
            die(Error::CannotWriteFile(file_path.display().to_string()));
        })
    }

    println!("{} changes", changes.len());
    for change in &changes {
        println!("Line {}: {}", change.line_number, change.change_type);
    }

    return changes.len();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_whitespace() {
        assert_eq!(is_file_whitespace(&[]), true);
        assert_eq!(is_file_whitespace(b"    "), true);
        assert_eq!(is_file_whitespace(b"\n\n\n"), true);
        assert_eq!(is_file_whitespace(b"\r\r\r"), true);
        assert_eq!(is_file_whitespace(b" \t\n\r"), true);
        assert_eq!(is_file_whitespace(b"hello"), false);
        assert_eq!(is_file_whitespace(b"hello world\n"), false);
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
    }

    #[test]
    fn test_modify_content_do_nothing() {
        let options: Options = Options::new();
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  ");
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_modify_content_do_nothing_whitespace_only_file() {
        let options: Options = Options::new();
        let mut output = Vec::new();
        let changes = modify_content(b"  ", &options, &mut output);
        assert_eq!(output, b"  ");
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_modify_content_do_nothing_empty_file() {
        let options: Options = Options::new();
        let mut output = Vec::new();
        let changes = modify_content(b"", &options, &mut output);
        assert_eq!(output, b"");
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_modify_content_add_new_line_marker_auto() {
        let options: Options = Options::new().add_new_line_marker_at_end_of_file();
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  \r");
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_modify_content_add_new_line_marker_linux() {
        let options: Options = Options::new()
            .add_new_line_marker_at_end_of_file()
            .new_line_marker(OutputNewLineMarkerMode::Linux);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  \n");
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_modify_content_add_new_line_marker_macos() {
        let options: Options = Options::new()
            .add_new_line_marker_at_end_of_file()
            .new_line_marker(OutputNewLineMarkerMode::MacOs);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  \r");
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_modify_content_add_new_line_marker_windows() {
        let options: Options = Options::new()
            .add_new_line_marker_at_end_of_file()
            .new_line_marker(OutputNewLineMarkerMode::Windows);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  ", &options, &mut output);
        assert_eq!(output, b"hello\r\n\rworld  \r\n");
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_modify_content_normalize_new_line_markers_auto() {
        let options: Options = Options::new().normalize_new_line_markers();
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  \r\n", &options, &mut output);
        assert_eq!(output, b"hello\r\n\r\nworld  \r\n");
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_modify_content_normalize_new_line_markers_linux() {
        let options: Options = Options::new()
            .normalize_new_line_markers()
            .new_line_marker(OutputNewLineMarkerMode::Linux);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  \r\n", &options, &mut output);
        assert_eq!(output, b"hello\n\nworld  \n");
        assert_eq!(changes.len(), 3);
    }

    #[test]
    fn test_modify_content_normalize_new_line_markers_macos() {
        let options: Options = Options::new()
            .normalize_new_line_markers()
            .new_line_marker(OutputNewLineMarkerMode::MacOs);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  \r\n", &options, &mut output);
        assert_eq!(output, b"hello\r\rworld  \r");
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn test_modify_content_normalize_new_line_markers_windows() {
        let options: Options = Options::new()
            .normalize_new_line_markers()
            .new_line_marker(OutputNewLineMarkerMode::Windows);
        let mut output = Vec::new();
        let changes = modify_content(b"hello\r\n\rworld  \r\n", &options, &mut output);
        assert_eq!(output, b"hello\r\n\r\nworld  \r\n");
        assert_eq!(changes.len(), 1);
    }
}
