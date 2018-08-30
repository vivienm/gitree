use std::fmt;

pub struct Report {
    num_dirs: usize,
    num_files: usize,
}

impl Report {
    pub fn new() -> Self {
        Report {
            num_dirs: 0,
            num_files: 0,
        }
    }

    pub fn add_dir(&mut self) {
        self.num_dirs += 1;
    }

    pub fn add_file(&mut self) {
        self.num_files += 1;
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} directories, {} files", self.num_dirs, self.num_files,)
    }
}
