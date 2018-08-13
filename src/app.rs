use clap::{App, Arg};

pub fn build_app() -> App<'static, 'static> {
    app_from_crate!()
        .arg(
            Arg::with_name("hidden")
                .long("hidden")
                .short("H")
                .help("Prints hidden files and directories"),
        )
        .arg(
            Arg::with_name("no_gitignore")
                .long("no-gitignore")
                .short("I")
                .help("Does not respect .gitignore files"),
        )
        .arg(
            Arg::with_name("follow_links")
                .long("--follow")
                .short("L")
                .help("Follows symbolic links"),
        )
        .arg(
            Arg::with_name("full_path")
                .long("full-path")
                .short("f")
                .help("Prints the full path prefix for each file"),
        )
        .arg(
            Arg::with_name("unsorted")
                .long("unsorted")
                .short("U")
                .help("Does not sort files"),
        )
        .arg(
            Arg::with_name("max_depth")
                .long("depth")
                .short("d")
                .takes_value(true)
                .value_name("DEPTH")
                .help("Sets maximum depth"),
        )
        .arg(
            Arg::with_name("exclude")
                .long("exclude")
                .short("E")
                .takes_value(true)
                .value_name("PATTERN")
                .help("Excludes files and directories that match the glob pattern"),
        )
        .arg(
            Arg::with_name("color")
                .long("color")
                .short("c")
                .takes_value(true)
                .value_name("WHEN")
                .possible_values(&["never", "auto", "always"])
                .help("Uses color for output"),
        )
        .arg(
            Arg::with_name("directory")
                .value_name("DIRECTORY")
                .multiple(true)
                .help("Directories to be listed"),
        )
}
