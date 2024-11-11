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
    ReplacedNonstandardWhitespaceBySpace,
    RemovedNonstandardWhitespace,
}

impl ChangeType {
    /// Human-readable representation of the change.
    pub fn to_string(&self, check_only: bool) -> String {
        let check_only_word = if check_only { " would be " } else { " " };
        match *self {
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
            ChangeType::ReplacedNewLineMarker(_, _) => {
                format!("New line marker '?'{}replaced by '?'.", check_only_word)
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
            ChangeType::ReplacedNonstandardWhitespaceBySpace => {
                format!(
                    "Non-standard whitespace character '?'{}replaced by a space.",
                    check_only_word
                )
            }
            ChangeType::RemovedNonstandardWhitespace => {
                format!(
                    "Non-standard whitespace character '?'{}removed.",
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
