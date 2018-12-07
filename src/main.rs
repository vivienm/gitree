mod app;
mod lscolors;
mod output;
mod pathtree;
mod report;
mod settings;
mod utils;

use std::error;
use std::io::{self, Write};
use std::path::Path;
use std::process;

use derive_more::{Display, From};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;

use crate::app::build_app;
use crate::output::write_tree_item;
use crate::pathtree::TreeBuilder;
use crate::report::Report;
use crate::settings::Settings;
use crate::utils::compare_file_names;

#[derive(Debug, Display, From)]
enum Error {
    Ignore(ignore::Error),
    Io(io::Error),
}

impl error::Error for Error {}

fn get_walk(path: &Path, settings: &Settings) -> Result<ignore::Walk, ignore::Error> {
    let mut walk_builder = WalkBuilder::new(path);
    walk_builder
        .hidden(settings.print_hidden)
        .parents(settings.print_ignored)
        .git_ignore(settings.print_ignored)
        .git_global(settings.print_ignored)
        .git_exclude(settings.print_ignored)
        .follow_links(settings.follow_links)
        .max_depth(settings.max_depth);

    if !settings.patterns.is_empty() {
        let mut override_builder = OverrideBuilder::new(path);
        override_builder.case_insensitive(settings.ignore_case)?;
        for pattern in &settings.patterns {
            override_builder.add(pattern)?;
        }
        let overrides = override_builder.build()?;
        walk_builder.overrides(overrides);
    }

    if settings.sort_files {
        walk_builder.sort_by_file_name(&compare_file_names);
    }

    Ok(walk_builder.build())
}

fn tree<'a, W>(output: &mut W, paths: Vec<&'a Path>, settings: &Settings) -> Result<(), Error>
where
    W: Write,
{
    let mut report = Report::new();
    for root_path in paths {
        let walk = get_walk(root_path, &settings)?;
        let direntries = walk.collect::<Result<Vec<_>, _>>()?;
        let tree = TreeBuilder::from_paths(&mut direntries.iter().map(|e| e.path()))
            .unwrap()
            .build();
        tree.for_each(&mut |item| write_tree_item(output, &mut report, item, &settings))?;
    }
    if settings.report {
        writeln!(output, "\n{}", report)?;
    }
    Ok(())
}

fn main() {
    let matches = build_app().get_matches();
    let settings = Settings::from_matches(&matches);
    let root_paths = match matches.values_of("directory") {
        Some(values) => values.map(Path::new).collect(),
        None => vec![Path::new(".")],
    };

    let mut stdout = io::stdout();
    match tree(&mut stdout, root_paths, &settings) {
        Ok(()) => process::exit(0),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(2);
        }
    }
}
