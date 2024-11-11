// Internal imports
use crate::error::print_error;

// Library imports
use clap;
use std::path::PathBuf;
use std::process;

/// A regular expression that does not match any string.
pub const UNMATCHABLE_REGEX: &str = "$.";

/// Color mode.
#[derive(clap::ValueEnum, Clone, PartialEq, Debug, Default)]
pub enum ColoredOutputMode {
    #[default]
    #[clap(help = "Detect coloring capabilities automatically.")]
    Auto,

    #[clap(help = "Turn off colored output.")]
    Off,

    #[clap(help = "Turn on colored output.")]
    On,
}

/// New line marker that should be used in the output files.
#[derive(clap::ValueEnum, Clone, PartialEq, Debug, Default)]
pub enum OutputNewLineMarkerMode {
    #[default]
    #[clap(
        help = "Use new line marker that is the most common in each individual file. \
        If no new line marker is present in the file, Linux '\\n' is used."
    )]
    Auto,

    #[clap(help = "Linux new line marker '\\n'.")]
    Linux,

    #[clap(help = "MacOS new line marker '\\r'.")]
    MacOs,

    #[clap(help = "Windows/DOS new line marker '\\r\\n'.")]
    Windows,
}

#[derive(clap::ValueEnum, Clone, PartialEq, Debug, Default)]
pub enum NonStandardWhitespaceReplacementMode {
    #[default]
    #[clap(help = "Leave '\\v' and '\\f' as is.")]
    Ignore,

    #[clap(help = "Replace any occurrence of `\\v` or `\\f` with a single space.")]
    ReplaceWithSpace,

    #[clap(help = "Remove all occurrences of `\\v' and '\\f'.")]
    Remove,
}

#[derive(clap::ValueEnum, Clone, PartialEq, Debug, Default)]
pub enum TrivialFileReplacementMode {
    #[default]
    #[clap(help = "Leave the file as is.")]
    Ignore,

    #[clap(help = "Replace the file with an empty file.")]
    Empty,

    #[clap(help = "Replace the file with a file consisting of a single new line marker.")]
    OneLine,
}

#[derive(clap::Parser, Debug)]
#[command(
    version,
    about = "Whitespace formatter and format checker for text files and source code files.",
    long_about = "Whitespace formatter and format checker for text files and source code files."
)]
pub struct CommandLineArguments {
    #[arg(
        long,
        default_value_t = false,
        help = "Do not format files. Only report which files would be formatted. \
        Exit code is zero if input is formatted correctly. Exit code is non-zero if formatting is required."
    )]
    pub check_only: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Follow symbolic links when searching for files."
    )]
    pub follow_symlinks: bool,

    #[arg(long,
    default_value_t = String::from(UNMATCHABLE_REGEX),
    help =
        "Regular expression that specifies which files to exclude. \
        The regular expression is evaluated on the path of each file. \
        The default value is a regular expression that does not match anything.",
    long_help =
        "Regular expression that specifies which files to exclude. \
        The regular expression is evaluated on the path of each file. \
        The default value is a regular expression that does not match anything.\
        \n\n\
        Example #1: --exclude=\"(.jpeg|.png)$\" excludes files with '.jpeg' or '.png' extension.\n\
        Example #2: --exclude=\".git/\" excludes all files in the '.git/' directory.\
    ")]
    pub exclude: String,

    #[arg(
        long,
        value_enum,
        default_value_t = ColoredOutputMode::Auto,
        help = "Enables or disables colored output."
    )]
    pub color: ColoredOutputMode,

    #[arg(long, value_enum,
    default_value_t = OutputNewLineMarkerMode::Auto,
    help = "New line marker to use.")]
    pub new_line_marker: OutputNewLineMarkerMode,

    #[arg(
        long,
        default_value_t = false,
        help = "Add a new line marker at the end of the file if it is missing."
    )]
    pub add_new_line_marker_at_end_of_file: bool,

    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "add_new_line_marker_at_end_of_file",
        help = "Remove all new line marker(s) from the end of each file. \
        This option conflicts with `--add-new-line-marker-at-end-of-file`. \
        This option implies `--remove-trailing-empty-lines` option, i.e., \
        all empty lines at the end of the file are removed."
    )]
    pub remove_new_line_marker_from_end_of_file: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Make new line markers the same within each file."
    )]
    pub normalize_new_line_markers: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Remove whitespace at the end of each line."
    )]
    pub remove_trailing_whitespace: bool,

    #[arg(
        long,
        default_value_t = false,
        default_value_if("remove_new_line_marker_from_end_of_file", "true", Some("true")),
        help = "Remove empty lines at the end of each file."
    )]
    pub remove_trailing_empty_lines: bool,

    #[arg(long,
    value_enum,
    default_value_t = TrivialFileReplacementMode::Ignore,
    help = "Replace files of zero length.",
    )]
    pub normalize_empty_files: TrivialFileReplacementMode,

    #[arg(long,
    value_enum,
    default_value_t = TrivialFileReplacementMode::Ignore,
    help = "Replace files consisting of whitespace only.")]
    pub normalize_whitespace_only_files: TrivialFileReplacementMode,

    #[arg(long,
    value_enum,
    default_value_t = NonStandardWhitespaceReplacementMode::Ignore,
    help = "Replace or remove non-standard whitespace characters '\\v' and '\\f' in each file.")]
    pub normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode,

    #[arg(long,
    default_value_t = -1,
    help = "Replace tabs with spaces. \
    The parameter specifies the number of spaces to use. \
    If the parameter is negative, tabs are not replaced.")]
    pub replace_tabs_with_spaces: isize,

    #[arg(num_args = 1..,
    required = true,
    value_delimiter = ' ',
    help = "List of files and/or directories to process. \
    Files in directories are discovered recursively.")]
    pub paths: Vec<PathBuf>,
}

impl CommandLineArguments {
    pub fn validate(&self) {
        if self.normalize_whitespace_only_files == TrivialFileReplacementMode::Empty
            && self.normalize_empty_files == TrivialFileReplacementMode::OneLine
        {
            print_error("the argument '--normalize-whitespace-only-files=empty' cannot be used with '--normalize-empty-files=one-line'");
            process::exit(1);
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    CommandLineArguments::command().debug_assert();
}
