use crate::core::char_to_str;
use crate::core::NewLineMarker;

/// Type of formatting change made in a file.
#[derive(PartialEq, Debug)]
pub enum ChangeType {
    /// New line marker was added to the end of the file (because it was missing).
    NewLineMarkerAddedToEndOfFile,

    /// New line marker was removed from the end of the file.
    NewLineMarkerRemovedFromEndOfFile,

    /// New line marker was replaced by another one.
    ReplacedNewLineMarker(NewLineMarker, NewLineMarker),

    /// White at the end of a line was removed.
    RemovedTrailingWhitespace,

    /// Empty line at the beginning of file was removed.
    RemovedLeadingEmptyLines,

    /// Empty line(s) at the end of file were removed.
    RemovedTrailingEmptyLines,

    /// An empty file was replaced by a file consisting of single empty line.
    ReplacedEmptyFileWithOneLine,

    /// A file consisting of only whitespace was replaced by an empty file.
    ReplacedWhiteSpaceOnlyFileWithEmptyFile,

    /// A file consisting of only whitespace was replaced by a file consisting of single empty line.
    ReplacedWhiteSpaceOnlyFileWithOneLine,

    /// A tab character was replaces by space character(s).
    ReplacedTabWithSpaces(isize),

    /// A tab character was removed.
    RemovedTab,

    /// A non-standard whitespace character ('\f' or '\v') was replaced by a space character.
    ReplacedNonstandardWhitespaceBySpace(u8),

    /// A non-standard whitespace character ('\f' or '\v') was removed.
    RemovedNonstandardWhitespace(u8),
}

impl ChangeType {
    /// Human-readable representation of the change.
    pub fn to_string(&self, check_only: bool) -> String {
        match self {
            ChangeType::NewLineMarkerAddedToEndOfFile => {
                if check_only {
                    "New line marker needs to be added at the end of the file.".to_string()
                } else {
                    "New line marker was added at the end of the file.".to_string()
                }
            }
            ChangeType::NewLineMarkerRemovedFromEndOfFile => {
                if check_only {
                    "New line marker needs to be removed from the end of the file.".to_string()
                } else {
                    "New line marker was removed from the end of the file.".to_string()
                }
            }
            ChangeType::ReplacedNewLineMarker(old, new) => {
                if check_only {
                    format!(
                        "New line marker '{}' needs to be replaced by '{}'.",
                        old, new
                    )
                } else {
                    format!("New line marker '{}' was replaced by '{}'.", old, new)
                }
            }
            ChangeType::RemovedTrailingWhitespace => {
                if check_only {
                    "Trailing whitespace needs to be removed.".to_string()
                } else {
                    "Trailing whitespace was removed.".to_string()
                }
            }
            ChangeType::RemovedLeadingEmptyLines => {
                if check_only {
                    "Empty lines at the beginning of the file need to be removed.".to_string()
                } else {
                    "Empty lines at the beginning of the file were removed.".to_string()
                }
            }
            ChangeType::RemovedTrailingEmptyLines => {
                if check_only {
                    "Empty lines at the end of the file need to be removed.".to_string()
                } else {
                    "Empty lines at the end of the file were removed.".to_string()
                }
            }
            ChangeType::ReplacedEmptyFileWithOneLine => {
                if check_only {
                    "Empty file needs to be replaced by single empty line.".to_string()
                } else {
                    "Empty file was replaced by a single empty line.".to_string()
                }
            }
            ChangeType::ReplacedWhiteSpaceOnlyFileWithEmptyFile => {
                if check_only {
                    "File consisting of only whitespace needs to be replaced by an empty file."
                        .to_string()
                } else {
                    "File consisting of only whitespace was replaced by an empty file.".to_string()
                }
            }
            ChangeType::ReplacedWhiteSpaceOnlyFileWithOneLine => {
                if check_only {
                    "File consisting of only whitespace needs to be replaced by a single empty line.".to_string()
                } else {
                    "File consisting of only whitespace was replaced by a single empty line."
                        .to_string()
                }
            }
            ChangeType::ReplacedTabWithSpaces(number_of_spaces) => {
                if check_only {
                    "Tab character needs to be replaced by spaces or removed.".to_string()
                } else {
                    format!(
                        "Tab character was replaced with {} spaces.",
                        number_of_spaces
                    )
                }
            }
            ChangeType::RemovedTab => {
                if check_only {
                    "Tab character needs to be replaced by spaces or removed.".to_string()
                } else {
                    "Tab character was removed.".to_string()
                }
            }
            ChangeType::ReplacedNonstandardWhitespaceBySpace(char) => {
                if check_only {
                    format!(
                        "Non-standard whitespace character '{}' needs to be replaced by a space.",
                        char_to_str(*char)
                    )
                } else {
                    format!(
                        "Non-standard whitespace character '{}' was replaced by a space.",
                        char_to_str(*char)
                    )
                }
            }
            ChangeType::RemovedNonstandardWhitespace(char) => {
                if check_only {
                    format!(
                        "Non-standard whitespace character '{}' needs to be removed.",
                        char_to_str(*char)
                    )
                } else {
                    format!(
                        "Non-standard whitespace character '{}' was removed.",
                        char_to_str(*char)
                    )
                }
            }
        }
    }
}

/// A formatting change that was made or would be made to a file.
/// The location of the change is identified by its line number.
#[derive(PartialEq, Debug)]
pub struct Change {
    line_number: usize,
    change_type: ChangeType,
}

impl Change {
    /// Constructor
    pub fn new(line_number: usize, change_type: ChangeType) -> Change {
        Change {
            line_number,
            change_type,
        }
    }

    /// Human-readable representation of the change
    pub fn to_string(&self, check_only: bool) -> String {
        format!(
            "line {}: {}",
            self.line_number,
            self.change_type.to_string(check_only)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_to_string() {
        assert_eq!(
            Change::new(1, ChangeType::NewLineMarkerAddedToEndOfFile).to_string(false),
            "line 1: New line marker was added at the end of the file."
        );
        assert_eq!(
            Change::new(1, ChangeType::NewLineMarkerAddedToEndOfFile).to_string(true),
            "line 1: New line marker needs to be added at the end of the file."
        );

        assert_eq!(
            Change::new(
                2,
                ChangeType::ReplacedNewLineMarker(NewLineMarker::Windows, NewLineMarker::Linux)
            )
            .to_string(false),
            "line 2: New line marker '\\r\\n' was replaced by '\\n'."
        );
        assert_eq!(
            Change::new(
                2,
                ChangeType::ReplacedNewLineMarker(NewLineMarker::Windows, NewLineMarker::Linux)
            )
            .to_string(true),
            "line 2: New line marker '\\r\\n' needs to be replaced by '\\n'."
        );

        assert_eq!(
            Change::new(3, ChangeType::ReplacedTabWithSpaces(4)).to_string(false),
            "line 3: Tab character was replaced with 4 spaces."
        );
        assert_eq!(
            Change::new(3, ChangeType::ReplacedTabWithSpaces(4)).to_string(true),
            "line 3: Tab character needs to be replaced by spaces or removed."
        );

        assert_eq!(
            Change::new(4, ChangeType::ReplacedNonstandardWhitespaceBySpace(0x0B)).to_string(false),
            "line 4: Non-standard whitespace character '\\v' was replaced by a space."
        );
        assert_eq!(
            Change::new(4, ChangeType::ReplacedNonstandardWhitespaceBySpace(0x0B)).to_string(true),
            "line 4: Non-standard whitespace character '\\v' needs to be replaced by a space."
        );

        assert_eq!(
            Change::new(5, ChangeType::RemovedNonstandardWhitespace(0x0C)).to_string(false),
            "line 5: Non-standard whitespace character '\\f' was removed."
        );
        assert_eq!(
            Change::new(5, ChangeType::RemovedNonstandardWhitespace(0x0C)).to_string(true),
            "line 5: Non-standard whitespace character '\\f' needs to be removed."
        );
    }
}
