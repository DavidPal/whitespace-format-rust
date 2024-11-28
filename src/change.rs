use crate::core::char_to_str;
use crate::core::NewLineMarker;

#[derive(PartialEq, Debug)]
pub enum ChangeType {
    NewLineMarkerAddedToEndOfFile,
    NewLineMarkerRemovedFromEndOfFile,
    ReplacedNewLineMarker(NewLineMarker, NewLineMarker),
    RemovedTrailingWhitespace,
    RemovedEmptyLines,
    ReplacedEmptyFileWithOneLine,
    ReplacedWhiteSpaceOnlyFileWithEmptyFile,
    ReplacedWhiteSpaceOnlyFileWithOneLine,
    ReplacedTabWithSpaces,
    RemovedTab,
    ReplacedNonstandardWhitespaceBySpace(u8),
    RemovedNonstandardWhitespace(u8),
}

impl ChangeType {
    /// Human-readable representation of the change.
    pub fn to_string(&self, check_only: bool) -> String {
        let check_only_word = if check_only { " would be " } else { " " };
        match self {
            ChangeType::NewLineMarkerAddedToEndOfFile => {
                format!(
                    "New line marker{}added to the end of the file.",
                    check_only_word
                )
            }
            ChangeType::NewLineMarkerRemovedFromEndOfFile => {
                format!(
                    "New line marker{}removed from the end of the file.",
                    check_only_word
                )
            }
            ChangeType::ReplacedNewLineMarker(old, new) => {
                format!(
                    "New line marker '{}'{}replaced by '{}'.",
                    old, check_only_word, new
                )
            }
            ChangeType::RemovedTrailingWhitespace => {
                format!("Trailing whitespace{}removed.", check_only_word)
            }
            ChangeType::RemovedEmptyLines => {
                format!(
                    "Empty line(s) at the end of the file{}removed.",
                    check_only_word
                )
            }
            ChangeType::ReplacedEmptyFileWithOneLine => {
                format!(
                    "Empty file{}replaced with a single empty line.",
                    check_only_word
                )
            }
            ChangeType::ReplacedWhiteSpaceOnlyFileWithEmptyFile => {
                format!("File{}replaced with an empty file.", check_only_word)
            }
            ChangeType::ReplacedWhiteSpaceOnlyFileWithOneLine => {
                format!("File{}replaced with a single empty line.", check_only_word)
            }
            ChangeType::ReplacedTabWithSpaces => {
                format!("Tab{}replaced with spaces.", check_only_word)
            }
            ChangeType::RemovedTab => {
                format!("Tab{}removed.", check_only_word)
            }
            ChangeType::ReplacedNonstandardWhitespaceBySpace(char) => {
                format!(
                    "Non-standard whitespace character '{}'{}replaced by a space.",
                    char_to_str(*char),
                    check_only_word
                )
            }
            ChangeType::RemovedNonstandardWhitespace(char) => {
                format!(
                    "Non-standard whitespace character '{}'{}removed.",
                    char_to_str(*char),
                    check_only_word
                )
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Change {
    pub line_number: usize,
    pub change_type: ChangeType,
}

impl Change {
    // Constructor
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
            "line 1: New line marker added to the end of the file."
        );

        assert_eq!(
            Change::new(
                2,
                ChangeType::ReplacedNewLineMarker(NewLineMarker::Windows, NewLineMarker::Linux)
            )
            .to_string(false),
            "line 2: New line marker '\\r\\n' replaced by '\\n'."
        );

        assert_eq!(
            Change::new(3, ChangeType::ReplacedNonstandardWhitespaceBySpace(0x0B)).to_string(false),
            "line 3: Non-standard whitespace character '\\v' replaced by a space."
        );

        assert_eq!(
            Change::new(4, ChangeType::RemovedNonstandardWhitespace(0x0C)).to_string(false),
            "line 4: Non-standard whitespace character '\\f' removed."
        );
    }
}
