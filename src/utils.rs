use std::env;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::process;

use lscolors::LsColors;

pub fn get_ls_colors() -> LsColors {
    env::var("GITREE_COLORS")
        .or(env::var("TREE_COLORS"))
        .or(env::var("LS_COLORS"))
        .ok()
        .map(|val| LsColors::from_string(&val))
        .unwrap_or_default()
}

pub fn error(message: &str) -> ! {
    eprintln!("{}", message);
    process::exit(1);
}

pub fn file_name_sort(file_name: &OsStr) -> Vec<u8> {
    let mut bytes = file_name.as_bytes();
    if *bytes.first().unwrap() == '.' as u8 {
        bytes = &bytes[1..];
    }
    bytes.to_ascii_lowercase()
}

#[test]
fn test_file_name_sort() {
    fn sorted_as(orig: &str, slug: &str) -> bool {
        file_name_sort(OsStr::new(orig)) == String::from(slug).into_bytes()
    }

    assert!(sorted_as("foobar", "foobar"));
    assert!(sorted_as("FooBar", "foobar"));
    assert!(sorted_as(".foobar", "foobar"));
    assert!(sorted_as("foo.bar", "foo.bar"));
}
