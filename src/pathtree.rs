use std::collections::hash_map;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct PathTree<'a> {
    pub roots: HashSet<&'a Path>,
    pub children: HashMap<&'a Path, HashSet<&'a Path>>,
}

impl<'a> PathTree<'a> {
    const MAP_CAPACITY: usize = 256;
    const SET_CAPACITY: usize = 16;

    pub fn with_roots<I>(roots: I) -> Self
    where
        I: Iterator<Item = &'a Path>,
    {
        PathTree {
            roots: roots.collect(),
            children: HashMap::with_capacity(Self::MAP_CAPACITY),
        }
    }

    pub fn insert(&mut self, path: &'a Path) {
        if self.roots.contains(path) {
            return;
        }
        let mut path = path;
        let ancestors = path.ancestors().skip(1);
        for parent in ancestors {
            match self.children.entry(parent) {
                hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().insert(path);
                    return;
                }
                hash_map::Entry::Vacant(entry) => {
                    let mut children = HashSet::with_capacity(Self::SET_CAPACITY);
                    children.insert(path);
                    entry.insert(children);
                    path = parent;
                }
            }
            if self.roots.contains(parent) {
                return;
            }
        }
    }

    pub fn _for_each(&self, func: &Fn(&Vec<bool>, &Path), prefixes: &mut Vec<bool>, path: &Path) {
        func(&prefixes, path);
        if let Some(children) = self.children.get(path) {
            let mut children: Vec<_> = children.iter().collect();
            children.sort();
            let (last_child, first_children) = children.split_last().unwrap();
            prefixes.push(false);
            for child in first_children {
                self._for_each(func, prefixes, child);
            }
            prefixes.pop();
            prefixes.push(true);
            self._for_each(func, prefixes, last_child);
            prefixes.pop();
        }
    }

    pub fn for_each(&self, func: &Fn(&Vec<bool>, &Path)) {
        let mut prefixes = Vec::new();
        let mut roots: Vec<_> = self.roots.iter().collect();
        roots.sort();
        for root in roots {
            self._for_each(func, &mut prefixes, root);
        }
    }
}
