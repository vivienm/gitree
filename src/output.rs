use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use ansi_term;

use lscolors::LsColors;
use pathtree::TreeItem;
use report::Report;
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

enum FileInfo {
    SymLink { target: PathBuf },
    Directory,
    File { executable: bool },
}

impl FileInfo {
    fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = path.symlink_metadata()?;
        if metadata.file_type().is_symlink() {
            let target = fs::read_link(path)?;
            Ok(FileInfo::SymLink { target })
        } else if metadata.is_dir() {
            Ok(FileInfo::Directory)
        } else {
            let executable = metadata.permissions().mode() & 0o111 != 0;
            Ok(FileInfo::File { executable })
        }
    }
}

fn get_path_style<'a>(
    path: &Path,
    info: &FileInfo,
    ls_colors: &'a LsColors,
) -> Option<&'a ansi_term::Style> {
    match info {
        FileInfo::SymLink { .. } => Some(&ls_colors.symlink),
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
}

fn write_file_label<'a, W>(
    output: &mut W,
    path: &Path,
    info: &FileInfo,
    ls_colors: Option<&'a LsColors>,
    print_path: bool,
) -> io::Result<()>
where
    W: Write,
{
    let label = if print_path {
        path.to_string_lossy()
    } else {
        path.file_name()
            .unwrap_or_else(|| OsStr::new(".."))
            .to_string_lossy()
    };
    let style = ls_colors.and_then(|ls_colors| get_path_style(path, info, ls_colors));
    if let Some(style) = style {
        write!(output, "{}", style.paint(label))?;
    } else {
        write!(output, "{}", label)?;
    }
    Ok(())
}

fn write_file_line<'a, W>(
    output: &mut W,
    report: &mut Report,
    path: &Path,
    ls_colors: Option<&'a LsColors>,
    print_path: bool,
) -> io::Result<()>
where
    W: Write,
{
    let info = FileInfo::from_path(path)?;
    write_file_label(output, path, &info, ls_colors, print_path)?;
    match info {
        FileInfo::Directory => report.add_dir(),
        FileInfo::File { .. } => report.add_file(),
        FileInfo::SymLink { target } => {
            let target_info = FileInfo::from_path(&target)?;
            write!(output, " -> ")?;
            write_file_label(output, path, &target_info, ls_colors, true)?;
            match target_info {
                FileInfo::Directory => report.add_dir(),
                FileInfo::File { .. } | FileInfo::SymLink { .. } => report.add_file(),
            }
        }
    };
    writeln!(output)
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
        write_file_line(output, report, item.path, ls_colors, settings.print_path)?;
    } else {
        write_file_line(output, report, item.path, ls_colors, true)?;
    }
    Ok(())
}
