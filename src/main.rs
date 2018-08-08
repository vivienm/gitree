extern crate ansi_term;
extern crate atty;
#[macro_use]
extern crate clap;
extern crate ignore;

mod app;
mod lscolors;
mod options;
mod output;
mod pathtree;

use std::env;
use std::path::Path;
use std::vec::Vec;

use atty::Stream;
use ignore::WalkBuilder;

use app::build_app;
use lscolors::LsColors;
use options::Options;
use output::print_entry;
use pathtree::PathTree;

fn get_ls_colors() -> LsColors {
    env::var("GITREE_COLORS")
        .or(env::var("TREE_COLORS"))
        .or(env::var("LS_COLORS"))
        .ok()
        .map(|val| LsColors::from_string(&val))
        .unwrap_or_default()
}

fn main() {
    let matches = build_app().get_matches();

    let colored_output = match matches.value_of("color") {
        Some("always") => true,
        Some("never") => false,
        _ => atty::is(Stream::Stdout),
    };
    let options = Options {
        ignore_hidden: !matches.is_present("hidden"),
        read_gitignore: !matches.is_present("no_gitignore"),
        follow_links: matches.is_present("follow_links"),
        max_depth: matches
            .value_of("max_depth")
            .and_then(|val| usize::from_str_radix(val, 10).ok()),
        ls_colors: {
            if colored_output {
                Some(get_ls_colors())
            } else {
                None
            }
        },
    };
    let root_paths = match matches.values_of("directory") {
        Some(values) => values.map(Path::new).collect(),
        None => vec![Path::new(".")],
    };

    let mut walk_builder = WalkBuilder::new(root_paths[0]);
    for root_path in root_paths.iter().skip(1) {
        walk_builder.add(root_path);
    }
    walk_builder
        .hidden(options.ignore_hidden)
        .follow_links(options.follow_links)
        .parents(options.read_gitignore)
        .git_ignore(options.read_gitignore)
        .git_global(options.read_gitignore)
        .git_exclude(options.read_gitignore)
        .max_depth(options.max_depth);
    let walk = walk_builder.build();

    let entries: Vec<_> = walk.filter_map(Result::ok).collect();
    let mut tree = PathTree::with_roots(root_paths.into_iter());
    entries.iter().for_each(|entry| {
        tree.insert(entry.path());
    });
    let tree = tree;

    tree.for_each(&move |prefixes, path| print_entry(prefixes, path, &options));
}
