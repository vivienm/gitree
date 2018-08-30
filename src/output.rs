use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use ansi_term;

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
    settings: &'a Settings,
) -> Option<&'a ansi_term::Style> {
    if let Some(ref ls_colors) = settings.ls_colors {
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
    } else {
        None
    }
}

fn write_file_line(
    output: &mut Write,
    report: &mut Report,
    path: &Path,
    label: &str,
    settings: &Settings,
) -> io::Result<()> {
    let info = FileInfo::from_path(path)?;
    match info {
        FileInfo::Directory => report.add_dir(),
        FileInfo::SymLink { .. } | FileInfo::File { .. } => report.add_file(),
    };
    if let Some(style) = get_path_style(path, &info, settings) {
        write!(output, "{}", style.paint(label))?;
    } else {
        write!(output, "{}", label)?;
    }
    if let FileInfo::SymLink { target } = info {
        write!(output, " -> {}", target.display())?;
    }
    writeln!(output)
}

fn write_path(
    output: &mut Write,
    report: &mut Report,
    path: &Path,
    settings: &Settings,
) -> io::Result<()> {
    write_file_line(output, report, path, &path.display().to_string(), settings)
}

fn write_file_name(
    output: &mut Write,
    report: &mut Report,
    path: &Path,
    settings: &Settings,
) -> io::Result<()> {
    write_file_line(
        output,
        report,
        path,
        &path.file_name().unwrap().to_string_lossy(),
        settings,
    )
}

pub fn write_tree_item(
    output: &mut Write,
    report: &mut Report,
    item: &TreeItem,
    settings: &Settings,
) -> io::Result<()> {
    if let Some((parent_indent, ancestor_indents)) = item.indents.split_last() {
        write_indents(&mut io::stdout(), ancestor_indents, *parent_indent).unwrap();
        if settings.print_path {
            write_path(output, report, item.path, settings)
        } else {
            write_file_name(output, report, item.path, settings)
        }
    } else {
        write_path(output, report, item.path, settings)
    }
}
