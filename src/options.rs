use lscolors::LsColors;

pub struct Options {
    pub ignore_hidden: bool,
    pub read_gitignore: bool,
    pub follow_links: bool,
    pub max_depth: Option<usize>,
    pub exclude_patterns: Vec<String>,
    pub ls_colors: Option<LsColors>,
    pub sort_files: bool,
}
