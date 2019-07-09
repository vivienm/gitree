use std::ffi::OsString;
use std::path::PathBuf;

use atty;
use clap;
use lscolors::LsColors;

use crate::utils::get_ls_colors;

pub struct Settings {
    // Print hidden files and directories.
    pub print_hidden: bool,

    // Print files ignored by Git.
    pub print_ignored: bool,

    // Custom ignore files.
    pub ignored_paths: Vec<PathBuf>,
    pub ignored_names: Vec<OsString>,

    // Follow symbolic links.
    pub follow_links: bool,

    // Print the full path prefix for each file.
    pub print_path: bool,

    // Maximum depth of the directory tree.
    pub max_depth: Option<usize>,

    // Glob patterns.
    pub patterns: Vec<String>,
    pub ignore_case: bool,

    // Sort files.
    pub sort_files: bool,

    // Report files and directories.
    pub report: bool,

    // Color codes.
    pub ls_colors: LsColors,
}

impl Settings {
    pub fn from_matches(matches: &clap::ArgMatches) -> Self {
        let colored_output = match matches.value_of("color") {
            Some("always") => true,
            Some("never") => false,
            _ => atty::is(atty::Stream::Stdout),
        };
        let mut patterns = vec![];
        for pattern in matches.values_of("patterns").unwrap_or_default() {
            patterns.push(String::from(pattern));
        }
        Settings {
            print_hidden: !matches.is_present("print_hidden"),
            print_ignored: !matches.is_present("print_ignored"),
            ignored_paths: matches
                .values_of("ignore_paths")
                .unwrap_or_default()
                .map(PathBuf::from)
                .collect(),
            ignored_names: matches
                .values_of("ignore_names")
                .unwrap_or_default()
                .map(OsString::from)
                .collect(),
            follow_links: matches.is_present("follow_links"),
            print_path: matches.is_present("print_path"),
            max_depth: matches
                .value_of("max_depth")
                .and_then(|val| usize::from_str_radix(val, 10).ok()),
            patterns,
            ignore_case: matches.is_present("ignore_case"),
            sort_files: !matches.is_present("no_sort_files"),
            report: !matches.is_present("no_report"),
            ls_colors: {
                if colored_output {
                    get_ls_colors()
                } else {
                    LsColors::empty()
                }
            },
        }
    }
}
