use std::path::PathBuf;

/// New line marker that should be used in the output files.
#[derive(clap::ValueEnum, Clone, PartialEq, Debug, Default)]
pub enum OutputNewLineMarkerMode {
    #[default]
    Auto,
    Linux,
    MacOs,
    Windows,
}

#[derive(clap::ValueEnum, Clone, PartialEq, Debug, Default)]
pub enum NonStandardWhitespaceReplacementMode {
    #[default]
    Ignore,
    ReplaceWithSpace,
    Remove,
}

#[derive(clap::ValueEnum, Clone, PartialEq, Debug, Default)]
pub enum TrivialFileReplacementMode {
    #[default]
    Ignore,
    Empty,
    OneLine,
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CommandLineArguments {
    #[arg(long, default_value_t = false)]
    pub add_new_line_marker_at_end_of_file: bool,

    #[arg(long, default_value_t = false)]
    pub remove_new_line_marker_from_end_of_file: bool,

    #[arg(long, default_value_t = false)]
    pub normalize_new_line_markers: bool,

    #[arg(long, default_value_t = false)]
    pub remove_trailing_whitespace: bool,

    #[arg(long, default_value_t = false)]
    pub remove_trailing_empty_lines: bool,

    #[arg(long, value_enum, default_value_t = OutputNewLineMarkerMode::Auto)]
    pub new_line_marker: OutputNewLineMarkerMode,

    #[arg(long, value_enum, default_value_t = TrivialFileReplacementMode::Ignore)]
    pub normalize_empty_files: TrivialFileReplacementMode,

    #[arg(long, value_enum, default_value_t = TrivialFileReplacementMode::Ignore)]
    pub normalize_whitespace_only_files: TrivialFileReplacementMode,

    #[arg(long, default_value_t = -1)]
    pub replace_tabs_with_spaces: isize,

    #[arg(long, value_enum, default_value_t = NonStandardWhitespaceReplacementMode::Ignore)]
    pub normalize_non_standard_whitespace: NonStandardWhitespaceReplacementMode,

    #[arg(num_args = 1.., required = true, value_delimiter = ' ')]
    pub paths: Vec<PathBuf>,
}
