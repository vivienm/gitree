use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use ansi_term;

use lscolors::LsColors;
use pathtree::TreeItem;
use settings::Settings;

const INDENT_EMPTY: &str = "    ";
const INDENT_BAR: &str = "│   ";
const INDENT_TEE: &str = "├── ";
const INDENT_ELL: &str = "└── ";

fn write_indents<W: Write>(
    output: &mut W,
    ancestor_indents: &[bool],
    parent_indent: bool,
) -> io::Result<()> {
    for ancestor_prefix in ancestor_indents {
        if *ancestor_prefix {
            write!(output, "{}", INDENT_EMPTY)?;
        } else {
            write!(output, "{}", INDENT_BAR)?;
        }
    }
    if parent_indent {
        write!(output, "{}", INDENT_ELL)?;
    } else {
        write!(output, "{}", INDENT_TEE)?;
    };
    Ok(())
}

#[test]
fn test_write_indents() {
    use std::io::Cursor;

    fn write_string(ancestor_indents: &[bool], parent_indent: bool) -> String {
        let mut buff = Cursor::new(Vec::new());
        write_indents(&mut buff, ancestor_indents, parent_indent).unwrap();
        String::from_utf8(buff.into_inner()).unwrap()
    }

    assert_eq!(write_string(&[], false), "├── ");
    assert_eq!(write_string(&[true, false], true), "    │   └── ");
}

fn is_symlink(md: &fs::Metadata) -> bool {
    md.file_type().is_symlink()
}

fn is_executable(md: &fs::Metadata) -> bool {
    md.permissions().mode() & 0o111 != 0
}

fn get_path_style<'a>(
    path: &Path,
    is_symlink: bool,
    ls_colors: &'a LsColors,
) -> Option<&'a ansi_term::Style> {
    if is_symlink {
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

fn write_file_line(
    output: &mut Write,
    path: &Path,
    label: &str,
    settings: &Settings,
) -> io::Result<()> {
    let is_symlink = path
        .symlink_metadata()
        .map(|md| is_symlink(&md))
        .unwrap_or(false);
    if let Some(ref ls_colors) = settings.ls_colors {
        if let Some(style) = get_path_style(path, is_symlink, ls_colors) {
            write!(output, "{}", style.paint(label))?;
        } else {
            write!(output, "{}", label)?;
        }
    } else {
        write!(output, "{}", label)?;
    }
    if is_symlink {
        if let Ok(target) = fs::read_link(path) {
            write!(output, " -> {}", target.display())?;
        }
    }
    writeln!(output)
}

fn write_path(output: &mut Write, path: &Path, settings: &Settings) -> io::Result<()> {
    write_file_line(output, path, &path.display().to_string(), settings)
}

fn write_file_name(output: &mut Write, path: &Path, settings: &Settings) -> io::Result<()> {
    write_file_line(
        output,
        path,
        &path.file_name().unwrap().to_string_lossy(),
        settings,
    )
}

pub fn write_tree_item(output: &mut Write, item: &TreeItem, settings: &Settings) -> io::Result<()> {
    if let Some((parent_indent, ancestor_indents)) = item.indents.split_last() {
        write_indents(&mut io::stdout(), ancestor_indents, *parent_indent).unwrap();
        if settings.print_path {
            write_path(output, item.path, settings)
        } else {
            write_file_name(output, item.path, settings)
        }
    } else {
        write_path(output, item.path, settings)
    }
}
