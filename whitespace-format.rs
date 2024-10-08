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
const WHITESPACE: u8 = b' ';
const TAB: u8 = b'\t';
const VERTICAL_TAB: u8 = b'\n'; // The same as '\v' in C, C++, Java and Python.
const FORM_FEED: u8 = 0x0C;  // The same as '\f' in C, C++, Java and Python.

// Possible line ending.
enum LineEnding {
    // Linux line ending is a single line feed character '\n'.
    Linux,

    // MacOS line ending is a single carriage return character '\r'.
    MacOs,

    // Windows/DOS line ending is a sequence of two characters:
    // carriage return character followed by line feed character.
    Windows,
}

impl Display for LineEnding {
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
    remove_trailing_empty_lines: bool,
    remove_trailing_whitespace: bool,
}

fn process_file(data: &[u8], options: &Options) {
    println!("The slice has {} bytes", data.len());

    for i in 0..data.len() {
        println!("{:02x} ", data[i]);
    }
}

/// Guesses line ending type based on content of the file.
/// The function computes the most common line ending that occurs in the file.
fn guess_line_ending(data: &[u8]) -> LineEnding {
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

    let max_count: usize = cmp::max(cmp::max(linux_count, macos_count), windows_count);

    if linux_count == max_count {
        return LineEnding::Linux;
    }
    if macos_count == max_count {
        return LineEnding::MacOs;
    }
    if windows_count == max_count {
        return LineEnding::Windows;
    }
    return LineEnding::Linux;
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

    let options: Options = Options {
        remove_trailing_empty_lines: false,
        remove_trailing_whitespace: false,
    };
    process_file(&data, &options);

    let line_ending: LineEnding = guess_line_ending(&data);
    println!("{}", line_ending);
}
