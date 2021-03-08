use std::io::{self, Write};
use std::path::Path;
use std::process;

use derive_more::{Display, Error, From};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use lscolors::LsColors;

use crate::cli::{self, IndentationMarks};
use crate::indent::{AsciiMarks, IndentationLevel, NullLevel, TreeLevel, UnicodeMarks};
use crate::output::write_tree_item;
use crate::pathtree::TreeBuilder;
use crate::report::Report;
use crate::utils::{compare_file_names, get_ls_colors};

#[derive(Debug, Display, From, Error)]
enum Error {
    Ignore(ignore::Error),
    Io(io::Error),
}

fn get_walk_builder(path: &Path, args: &cli::Args) -> Result<ignore::WalkBuilder, ignore::Error> {
    let mut walk_builder = WalkBuilder::new(path);
    walk_builder
        .hidden(args.print_hidden)
        .parents(args.print_ignored)
        .git_ignore(args.print_ignored)
        .git_global(args.print_ignored)
        .git_exclude(args.print_ignored)
        .follow_links(args.follow_links)
        .max_depth(args.max_depth)
        .same_file_system(args.same_file_system);

    for path in &args.ignore_paths {
        walk_builder.add_ignore(path);
    }
    for name in &args.ignore_names {
        walk_builder.add_custom_ignore_filename(name);
    }

    if !args.patterns.is_empty() {
        let mut override_builder = OverrideBuilder::new(path);
        override_builder.case_insensitive(args.ignore_case)?;
        for pattern in &args.patterns {
            override_builder.add(pattern)?;
        }
        let overrides = override_builder.build()?;
        walk_builder.overrides(overrides);
    }

    if args.sort_files {
        walk_builder.sort_by_file_name(&compare_file_names);
    }

    Ok(walk_builder)
}

fn get_walk(path: &Path, args: &cli::Args) -> Result<ignore::Walk, ignore::Error> {
    Ok(get_walk_builder(path, args)?.build())
}

impl cli::ColorMode {
    fn use_color(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}

fn write_tree<L, W>(output: &mut W, level: &mut L, args: &cli::Args) -> Result<(), Error>
where
    L: IndentationLevel,
    W: Write,
{
    let ls_colors = if args.color.use_color() {
        get_ls_colors()
    } else {
        LsColors::empty()
    };
    let mut report = Report::new();
    for root_path in &args.directories {
        let walk = get_walk(root_path, &args)?;
        let direntries = walk.collect::<Result<Vec<_>, _>>()?;
        let tree = TreeBuilder::from_paths(&mut direntries.iter().map(|e| e.path()))
            .unwrap()
            .build();
        tree.for_each(level, &mut |level, path| {
            write_tree_item(
                output,
                &mut report,
                level,
                path,
                &ls_colors,
                args.print_path,
            )
        })?;
    }
    if args.report {
        writeln!(output, "\n{}", report)?;
    }
    Ok(())
}

pub fn main(args: &cli::Args) {
    let mut stdout = io::stdout();
    let mut level: Box<dyn IndentationLevel> = match args.indentation {
        IndentationMarks::None => Box::new(NullLevel::new()),
        IndentationMarks::Ascii => Box::new(TreeLevel::<AsciiMarks>::new()),
        IndentationMarks::Unicode => Box::new(TreeLevel::<UnicodeMarks>::new()),
    };
    match write_tree(&mut stdout, &mut level, &args) {
        Ok(()) => process::exit(0),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(2);
        }
    }
}
