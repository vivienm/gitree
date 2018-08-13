extern crate ansi_term;
extern crate atty;
#[macro_use]
extern crate clap;
extern crate ignore;

mod app;
mod lscolors;
mod output;
mod pathtree;
mod settings;
mod utils;

use std::path::Path;

use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;

use app::build_app;
use output::print_entry;
use pathtree::TreeBuilder;
use settings::Settings;
use utils::{error, file_name_sort};

fn get_walk(path: &Path, settings: &Settings) -> ignore::Walk {
    let mut walk_builder = WalkBuilder::new(path);
    walk_builder
        .hidden(settings.print_hidden)
        .parents(settings.print_ignored)
        .git_ignore(settings.print_ignored)
        .git_global(settings.print_ignored)
        .git_exclude(settings.print_ignored)
        .follow_links(settings.follow_links)
        .max_depth(settings.max_depth);

    if !settings.include_patterns.is_empty() {
        let mut override_builder = OverrideBuilder::new(path);
        override_builder
            .case_insensitive(settings.ignore_case)
            .unwrap();
        for pattern in &settings.include_patterns {
            override_builder.add(pattern).unwrap_or_else(|err| {
                error(&format!("Invalid pattern {:?}: {}", pattern, err));
            });
        }
        let overrides = override_builder.build().unwrap_or_else(|err| {
            error(&format!("Invalid patterns: {}", err));
        });
        walk_builder.overrides(overrides);
    }

    if settings.sort_files {
        walk_builder.sort_by_file_name(|fn1, fn2| {
            let fn1 = file_name_sort(fn1);
            let fn2 = file_name_sort(fn2);
            fn1.cmp(&fn2)
        });
    }

    walk_builder.build()
}

fn main() {
    let matches = build_app().get_matches();
    let settings = Settings::from_matches(&matches);
    let root_paths = match matches.values_of("directory") {
        Some(values) => values.map(Path::new).collect(),
        None => vec![Path::new(".")],
    };

    for root_path in root_paths {
        let walk = get_walk(root_path, &settings);
        let direntries: Vec<_> = walk.filter_map(|e| e.ok()).collect();
        let tree = TreeBuilder::from_paths(&mut direntries.iter().map(|e| e.path()))
            .unwrap()
            .build();
        tree.for_each(&|prefixes, path| print_entry(prefixes, path, &settings));
    }
}
