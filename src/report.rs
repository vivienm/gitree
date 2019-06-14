use std::fmt;
use std::fs::FileType;

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

    pub fn add(&mut self, toplevel: bool, file_type: &FileType) {
        if !toplevel {
            if file_type.is_dir() {
                self.num_dirs += 1;
            } else {
                self.num_files += 1;
            }
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}, {} {}",
            self.num_dirs,
            if self.num_dirs == 1 {
                "directory"
            } else {
                "directories"
            },
            self.num_files,
            if self.num_files == 1 { "file" } else { "files" }
        )
    }
}
