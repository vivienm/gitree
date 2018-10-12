use clap::{App, Arg};

pub fn build_app() -> App<'static, 'static> {
    app_from_crate!()
        .arg(
            Arg::with_name("print_hidden")
                .long("hidden")
                .short("H")
                .help("Prints hidden files and directories"),
        ).arg(
            Arg::with_name("print_ignored")
                .long("no-ignore")
                .short("I")
                .help("Prints files and directories ignored by Git"),
        ).arg(
            Arg::with_name("follow_links")
                .long("follow")
                .short("L")
                .help("Follows symbolic links"),
        ).arg(
            Arg::with_name("print_path")
                .long("full-path")
                .short("p")
                .help("Prints the full path of each file and directory"),
        ).arg(
            Arg::with_name("max_depth")
                .long("max-depth")
                .short("d")
                .takes_value(true)
                .value_name("LEVEL")
                .help("Maximum depth of the directory tree"),
        ).arg(
            Arg::with_name("patterns")
                .long("glob")
                .short("g")
                .takes_value(true)
                .value_name("PATTERN")
                .multiple(true)
                .number_of_values(1)
                .help("Includes or excludes files and directories that match the glob pattern"),
        ).arg(
            Arg::with_name("ignore_case")
                .long("ignore-case")
                .short("i")
                .help("Performs case-insensitive pattern matching"),
        ).arg(
            Arg::with_name("no_sort_files")
                .long("no-sort-files")
                .short("S")
                .help("Does not sort files"),
        ).arg(
            Arg::with_name("no_report")
                .long("no-report")
                .short("R")
                .help("Does not print the report"),
        ).arg(
            Arg::with_name("color")
                .long("color")
                .short("c")
                .takes_value(true)
                .value_name("WHEN")
                .possible_values(&["never", "auto", "always"])
                .help("Uses color for output"),
        ).arg(
            Arg::with_name("directory")
                .value_name("DIRECTORY")
                .multiple(true)
                .help("Directories to display"),
        )
}
