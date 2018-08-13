use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use ansi_term;

use lscolors::LsColors;
use settings::Settings;

fn is_executable(md: &fs::Metadata) -> bool {
    md.permissions().mode() & 0o111 != 0
}

fn get_path_style<'a>(path: &Path, ls_colors: &'a LsColors) -> Option<&'a ansi_term::Style> {
    if path
        .symlink_metadata()
        .map(|md| md.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Some(&ls_colors.symlink);
    }

    let metadata = path.metadata();

    if metadata.as_ref().map(|md| md.is_dir()).unwrap_or(false) {
        Some(&ls_colors.directory)
    } else if metadata.map(|md| is_executable(&md)).unwrap_or(false) {
        Some(&ls_colors.executable)
    } else if let Some(filename_style) = path
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| ls_colors.filenames.get(n))
    {
        Some(filename_style)
    } else if let Some(extension_style) = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(|e| ls_colors.extensions.get(e))
    {
        Some(extension_style)
    } else {
        None
    }
}

const PREFIX_EMPTY: &'static str = "    ";
const PREFIX_VERT: &'static str = "│   ";
const PREFIX_TEE: &'static str = "├── ";
const PREFIX_LAST: &'static str = "└── ";

fn print_prefixes(ancestor_prefixes: &[bool], parent_prefix: &bool) {
    for ancestor_prefix in ancestor_prefixes {
        if *ancestor_prefix {
            print!("{}", PREFIX_EMPTY);
        } else {
            print!("{}", PREFIX_VERT);
        }
    }
    if *parent_prefix {
        print!("{}", PREFIX_LAST);
    } else {
        print!("{}", PREFIX_TEE);
    }
}

fn print_file(label: &str, path: &Path, options: &Settings) {
    if let Some(ref ls_colors) = options.ls_colors {
        let default_style = ansi_term::Style::default();
        let style = get_path_style(path, ls_colors).unwrap_or(&default_style);
        println!("{}", style.paint(label))
    } else {
        println!("{}", label)
    }
}

fn print_path(path: &Path, options: &Settings) {
    print_file(&path.display().to_string(), path, options);
}

fn print_file_name(path: &Path, options: &Settings) {
    print_file(&path.file_name().unwrap().to_string_lossy(), path, options);
}

pub fn print_entry(prefixes: &[bool], path: &Path, options: &Settings) {
    if let Some((parent_prefix, ancestor_prefixes)) = prefixes.split_last() {
        print_prefixes(ancestor_prefixes, parent_prefix);
        if options.print_path {
            print_path(path, options);
        } else {
            print_file_name(path, options);
        }
    } else {
        print_path(path, options);
    }
}
