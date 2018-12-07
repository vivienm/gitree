use std::borrow;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use ansi_term;

use crate::lscolors::LsColors;
use crate::pathtree::TreeItem;
use crate::report::Report;
use crate::settings::Settings;

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

enum FileInfo {
    SymLink {
        target: PathBuf,
        resolved: Option<PathBuf>,
    },
    Directory,
    File {
        executable: bool,
    },
}

impl FileInfo {
    fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = path.symlink_metadata()?;
        let file_info = if metadata.file_type().is_symlink() {
            let target = fs::read_link(path)?;
            let resolved = match fs::canonicalize(path) {
                Ok(resolved) => Some(resolved),
                Err(ref err) if err.kind() == io::ErrorKind::NotFound => None,
                Err(err) => return Err(err),
            };
            FileInfo::SymLink { target, resolved }
        } else if metadata.is_dir() {
            FileInfo::Directory
        } else {
            let executable = metadata.permissions().mode() & 0o111 != 0;
            FileInfo::File { executable }
        };
        Ok(file_info)
    }
}

fn get_path_style<'a>(
    path: &Path,
    info: &FileInfo,
    ls_colors: Option<&'a LsColors>,
) -> Option<&'a ansi_term::Style> {
    if let Some(ls_colors) = ls_colors {
        match info {
            FileInfo::SymLink {
                resolved: Some(..), ..
            } => Some(&ls_colors.symlink),
            FileInfo::SymLink { resolved: None, .. } => Some(&ls_colors.orphan),
            FileInfo::Directory => Some(&ls_colors.directory),
            FileInfo::File { executable: true } => Some(&ls_colors.executable),
            FileInfo::File { executable: false } => {
                if let Some(filename_style) = path
                    .file_name()
                    .and_then(|filename| filename.to_str())
                    .and_then(|filename| ls_colors.filenames.get(filename))
                {
                    Some(filename_style)
                } else if let Some(extension_style) = path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .and_then(|extension| ls_colors.extensions.get(extension))
                {
                    Some(extension_style)
                } else {
                    None
                }
            }
        }
    } else {
        None
    }
}

fn get_path_label(path: &Path, print_path: bool) -> borrow::Cow<str> {
    if print_path {
        path.to_string_lossy()
    } else {
        path.file_name()
            .unwrap_or_else(|| OsStr::new(".."))
            .to_string_lossy()
    }
}

fn write_path_label<W>(
    output: &mut W,
    path: &Path,
    style: Option<&ansi_term::Style>,
    print_path: bool,
) -> io::Result<()>
where
    W: Write,
{
    let label = get_path_label(path, print_path);
    if let Some(style) = style {
        write!(output, "{}", style.paint(label))?;
    } else {
        write!(output, "{}", label)?;
    }
    Ok(())
}

fn write_file_line<'a, W>(
    output: &mut W,
    report: Option<&mut Report>,
    path: &Path,
    ls_colors: Option<&'a LsColors>,
    print_path: bool,
) -> io::Result<()>
where
    W: Write,
{
    let info = FileInfo::from_path(path)?;
    let style = get_path_style(path, &info, ls_colors);
    write_path_label(output, path, style, print_path)?;
    match info {
        FileInfo::Directory => report.map(Report::add_dir),
        FileInfo::File { .. } => report.map(Report::add_file),
        FileInfo::SymLink {
            target: target_path,
            resolved: resolved_path,
        } => {
            write!(output, " -> ")?;
            match resolved_path {
                None => {
                    write_path_label(output, &target_path, style, true)?;
                    report.map(Report::add_file)
                }
                Some(resolved_path) => {
                    let resolved_info = FileInfo::from_path(&resolved_path)?;
                    let resolved_style = get_path_style(&resolved_path, &resolved_info, ls_colors);
                    write_path_label(output, &target_path, resolved_style, true)?;
                    match resolved_info {
                        FileInfo::Directory => report.map(Report::add_dir),
                        FileInfo::File { .. } | FileInfo::SymLink { .. } => {
                            report.map(Report::add_file)
                        }
                    }
                }
            }
        }
    };
    writeln!(output)?;
    Ok(())
}

pub fn write_tree_item<W>(
    output: &mut W,
    report: &mut Report,
    item: &TreeItem,
    settings: &Settings,
) -> io::Result<()>
where
    W: Write,
{
    let ls_colors = settings.ls_colors.as_ref();
    if let Some((parent_indent, ancestor_indents)) = item.indents.split_last() {
        write_indents(output, ancestor_indents, *parent_indent)?;
        write_file_line(
            output,
            Some(report),
            item.path,
            ls_colors,
            settings.print_path,
        )?;
    } else {
        write_file_line(output, None, item.path, ls_colors, true)?;
    }
    Ok(())
}
