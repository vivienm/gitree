use std::env;
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
