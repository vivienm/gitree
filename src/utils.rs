use std::cmp::Ordering;
use std::env;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use crate::lscolors::LsColors;

pub fn get_ls_colors() -> LsColors {
    env::var("GITREE_COLORS")
        .or_else(|_| env::var("TREE_COLORS"))
        .or_else(|_| env::var("LS_COLORS"))
        .ok()
        .map(|val| LsColors::from_string(&val))
        .unwrap_or_default()
}

pub fn compare_file_names(file_name_1: &OsStr, file_name_2: &OsStr) -> Ordering {
    let mut bytes_1 = file_name_1.as_bytes().iter();
    let mut bytes_2 = file_name_2.as_bytes().iter();
    // Strip initial dot.
    let mut byte_1_opt = bytes_1.next();
    let mut byte_2_opt = bytes_2.next();
    if *byte_1_opt.unwrap() == b'.' {
        byte_1_opt = bytes_1.next();
    }
    if *byte_2_opt.unwrap() == b'.' {
        byte_2_opt = bytes_2.next();
    }
    loop {
        match (byte_1_opt, byte_2_opt) {
            (None, None) => {
                return Ordering::Equal;
            }
            (None, Some(_)) => {
                return Ordering::Less;
            }
            (Some(_), None) => {
                return Ordering::Greater;
            }
            (Some(byte_1), Some(byte_2)) => {
                let byte_1 = byte_1.to_ascii_lowercase();
                let byte_2 = byte_2.to_ascii_lowercase();
                if byte_1 < byte_2 {
                    return Ordering::Less;
                } else if byte_1 > byte_2 {
                    return Ordering::Greater;
                } else {
                    byte_1_opt = bytes_1.next();
                    byte_2_opt = bytes_2.next();
                }
            }
        }
    }
}

#[test]
fn test_compare_file_names() {
    fn compare_str(file_name_1: &str, file_name_2: &str) -> Ordering {
        compare_file_names(OsStr::new(file_name_1), OsStr::new(file_name_2))
    }

    // Trivial equality.
    assert_eq!(compare_str("foobar", "foobar"), Ordering::Equal);

    // Ignore leading dot.
    assert_eq!(compare_str("foobar", ".foobar"), Ordering::Equal);
    assert_eq!(compare_str("foobar", "..foobar"), Ordering::Greater);

    // Ignore case.
    assert_eq!(compare_str("foobar", "FooBar"), Ordering::Equal);

    // Size matters.
    assert_eq!(compare_str("foo", "foobar"), Ordering::Less);
}
