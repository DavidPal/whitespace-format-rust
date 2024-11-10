use std::fmt;

#[derive(PartialEq, Debug)]
pub enum ChangeType {
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

pub struct Change {
    pub line_number: usize,
    pub change_type: ChangeType,
}
