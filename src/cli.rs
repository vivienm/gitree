use std::ffi::OsString;
use std::fmt;
use std::ops;
use std::path::PathBuf;
use std::str::FromStr;

use structopt::StructOpt;

#[derive(Debug)]
pub enum ColorMode {
    Always,
    Never,
    Auto,
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::Auto
    }
}

impl FromStr for ColorMode {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            "auto" => Ok(Self::Auto),
            _ => Err("valid values: always, never, auto"),
        }
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

impl ColorMode {
    pub fn variants() -> [&'static str; 3] {
        ["always", "never", "auto"]
    }
}

#[derive(Debug)]
pub enum IndentationMarks {
    Ascii,
    Unicode,
    None,
}

impl Default for IndentationMarks {
    fn default() -> Self {
        Self::Unicode
    }
}

impl FromStr for IndentationMarks {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ascii" => Ok(Self::Ascii),
            "unicode" => Ok(Self::Unicode),
            "none" => Ok(Self::None),
            _ => Err("valid values: ascii, unicode, none"),
        }
    }
}

impl fmt::Display for IndentationMarks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ascii => write!(f, "ascii"),
            Self::Unicode => write!(f, "unicode"),
            Self::None => write!(f, "none"),
        }
    }
}

impl IndentationMarks {
    fn variants() -> [&'static str; 3] {
        ["ascii", "unicode", "none"]
    }
}

/// Print a directory tree while respecting gitignore rules
#[derive(Debug, StructOpt)]
#[structopt(global_setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct Args {
    /// Prints hidden files and directories
    #[structopt(short = "H", long = "hidden")]
    pub print_hidden: bool,
    /// Prints files and directories ignored by Git
    #[structopt(short = "I", long = "no-ignore")]
    pub print_ignored: bool,
    /// Adds a custom ignore filepath in gitignore format
    #[structopt(
        long = "ignore-path",
        value_name = "PATH",
        multiple = true,
        number_of_values = 1,
        parse(from_os_str)
    )]
    pub ignore_paths: Vec<PathBuf>,
    /// Adds a custom ignore filename in gitignore format
    #[structopt(
        long = "ignore-name",
        value_name = "NAME",
        multiple = true,
        number_of_values = 1,
        parse(from_os_str)
    )]
    pub ignore_names: Vec<OsString>,
    /// Follows symbolic links
    #[structopt(short = "L", long = "follow")]
    pub follow_links: bool,
    /// Prints the full path of each file and directory
    #[structopt(short = "p", long = "full-path")]
    pub print_path: bool,
    /// Maximum depth of the directory tree
    #[structopt(short = "d", long = "max-depth", value_name = "LEVEL")]
    pub max_depth: Option<usize>,
    /// Stays on the current filesystem only
    #[structopt(short = "x", long = "one-file-system")]
    pub same_file_system: bool,
    /// Includes or excludes files and directories that match the glob pattern
    #[structopt(
        short = "g",
        long = "glob",
        value_name = "PATTERN",
        multiple = true,
        number_of_values = 1
    )]
    pub patterns: Vec<String>,
    /// Performs case-insensitive pattern matching
    #[structopt(short = "i", long = "ignore-case")]
    pub ignore_case: bool,
    /// Does not sort files
    #[structopt(short = "S", long = "no-sort-files", parse(from_flag = ops::Not::not))]
    pub sort_files: bool,
    /// Does not print the report
    #[structopt(short = "R", long = "no-report", parse(from_flag = ops::Not::not))]
    pub report: bool,
    /// Uses colors for output
    #[structopt(
        short = "c",
        long = "color",
        value_name= " MODE",
        default_value = "auto",
        possible_values=&ColorMode::variants()
    )]
    pub color: ColorMode,
    /// Indentation lines
    #[structopt(
        long="indentation",
        value_name = "TYPE",
        default_value = "unicode",
        possible_values = &IndentationMarks::variants()
    )]
    pub indentation: IndentationMarks,
    /// Directories to display
    #[structopt(
        value_name = "DIRECTORY",
        multiple = true,
        default_value = ".",
        parse(from_os_str)
    )]
    pub directories: Vec<PathBuf>,
}
