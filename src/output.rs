use std::borrow;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use ansi_term;
use lscolors::{LsColors, Style};

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
    ls_colors: &'a LsColors,
    print_path: bool,
) -> io::Result<()>
where
    W: Write,
{
    let metadata = path.symlink_metadata()?;
    let style = ls_colors
        .style_for_path_with_metadata(path, Some(&metadata))
        .map(Style::to_ansi_term_style);
    write_path_label(output, path, style.as_ref(), print_path)?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        write!(output, " -> ")?;
        let relative_target = fs::read_link(path)?;
        match fs::canonicalize(relative_target) {
            Ok(absolute_target) => {
                let target_metadata = absolute_target.symlink_metadata()?;
                let target_style = ls_colors
                    .style_for_path_with_metadata(path, Some(&metadata))
                    .map(Style::to_ansi_term_style);
                write_path_label(output, path, target_style.as_ref(), print_path)?;
                report.map(if target_metadata.is_dir() {
                    Report::add_dir
                } else {
                    Report::add_file
                });
            }
            Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
                write_path_label(output, path, style.as_ref(), print_path)?;
                report.map(Report::add_file);
            }
            Err(err) => return Err(err),
        };
    } else if file_type.is_dir() {
        report.map(Report::add_dir);
    } else {
        report.map(Report::add_file);
    }
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
    if let Some((parent_indent, ancestor_indents)) = item.indents.split_last() {
        write_indents(output, ancestor_indents, *parent_indent)?;
        write_file_line(
            output,
            Some(report),
            item.path,
            &settings.ls_colors,
            settings.print_path,
        )?;
    } else {
        write_file_line(output, None, item.path, &settings.ls_colors, true)?;
    }
    Ok(())
}
