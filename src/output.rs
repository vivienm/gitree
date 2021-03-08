use std::borrow;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use lscolors::{LsColors, Style};

use crate::indent::IndentationLevel;
use crate::report::Report;

fn get_path_label(path: &Path, print_path: bool) -> borrow::Cow<str> {
    if print_path {
        path.to_string_lossy()
    } else {
        path.file_name()
            .map(OsStr::to_string_lossy)
            .unwrap_or_else(|| "..".into())
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
    toplevel: bool,
    output: &mut W,
    report: &mut Report,
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
        match fs::canonicalize(&relative_target) {
            Ok(absolute_target) => {
                let target_metadata = absolute_target.symlink_metadata()?;
                let target_style = ls_colors
                    .style_for_path_with_metadata(path, Some(&target_metadata))
                    .map(Style::to_ansi_term_style);
                write_path_label(
                    output,
                    relative_target.as_path(),
                    target_style.as_ref(),
                    true,
                )?;
                report.add(toplevel, target_metadata.file_type());
            }
            Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
                write_path_label(output, relative_target.as_path(), style.as_ref(), true)?;
                report.add(toplevel, file_type);
            }
            Err(err) => return Err(err),
        };
    } else {
        report.add(toplevel, file_type);
    }
    writeln!(output)?;
    Ok(())
}

pub fn write_tree_item<'a, L, W>(
    output: &mut W,
    report: &mut Report,
    level: &L,
    path: &Path,
    ls_colors: &'a LsColors,
    print_path: bool,
) -> io::Result<()>
where
    L: IndentationLevel,
    W: Write,
{
    write!(output, "{}", level)?;
    let toplevel = level.is_empty();
    write_file_line(
        toplevel,
        output,
        report,
        path,
        ls_colors,
        toplevel || print_path,
    )?;
    Ok(())
}
