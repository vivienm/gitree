use std::path::Path;

use crate::indent::IndentationLevel;

pub type TreeIndex = usize;

pub struct TreeNode<'a> {
    path: &'a Path,
    children: Vec<TreeIndex>,
}

impl<'a> TreeNode<'a> {
    fn new(path: &'a Path, children: Vec<TreeIndex>) -> Self {
        TreeNode { path, children }
    }
}

pub struct Tree<'a> {
    nodes: Vec<TreeNode<'a>>,
}

impl<'a> Tree<'a> {
    const ROOT: TreeIndex = 0;

    #[inline]
    pub fn get_node(&self, index: TreeIndex) -> &TreeNode<'a> {
        &self.nodes[index]
    }

    fn _for_each<E, F, L>(&self, func: &mut F, level: &mut L, node: &TreeNode) -> Result<(), E>
    where
        F: FnMut(&L, &Path) -> Result<(), E>,
        L: IndentationLevel,
    {
        func(level, node.path)?;
        if let Some((last_index, first_indices)) = node.children.split_last() {
            level.indent();
            for child_index in first_indices {
                let child_node = &self.get_node(*child_index);
                self._for_each(func, level, child_node)?;
            }
            level.set_last();
            let child_node = &self.get_node(*last_index);
            self._for_each(func, level, child_node)?;
            level.dedent();
        }
        Ok(())
    }

    pub fn for_each<E, F, L>(&self, level: &mut L, func: &mut F) -> Result<(), E>
    where
        F: FnMut(&L, &Path) -> Result<(), E>,
        L: IndentationLevel,
    {
        let root_node = &self.get_node(Self::ROOT);
        self._for_each(func, level, root_node)
    }
}

pub struct TreeBuilder<'a> {
    nodes: Vec<TreeNode<'a>>,
    root_depth: usize,
    indices: Vec<TreeIndex>,
}

impl<'a> TreeBuilder<'a> {
    pub fn build(self) -> Tree<'a> {
        Tree { nodes: self.nodes }
    }

    fn with_root(root_path: &'a Path) -> TreeBuilder<'a> {
        let root_node = TreeNode::new(root_path, vec![]);
        let root_depth = root_node.path.components().count();
        TreeBuilder {
            nodes: vec![root_node],
            root_depth,
            indices: vec![0],
        }
    }

    pub fn from_paths<I>(paths: &mut I) -> Option<TreeBuilder<'a>>
    where
        I: Iterator<Item = &'a Path>,
    {
        let mut builder = match paths.next() {
            None => {
                return None;
            }
            Some(root_path) => Self::with_root(root_path),
        };
        for path in paths {
            builder.push(path);
        }
        Some(builder)
    }

    fn push(&mut self, path: &'a Path) {
        // At his point, we know that path is not a root.

        let extra_depth = {
            // Compute the number of common components from root.
            let prev_node = &self.nodes[*self.indices.last().unwrap()];
            let mut path_components = path.components().skip(self.root_depth);
            let mut prev_components = prev_node.path.components().skip(self.root_depth);
            let mut num_shared_components = 0;
            loop {
                match (prev_components.next(), path_components.next()) {
                    (_, None) => {
                        // Should not happend if insertion order is correct.
                        unreachable!();
                    }
                    (Some(prev_component), Some(path_component)) => {
                        if prev_component == path_component {
                            num_shared_components += 1;
                        } else {
                            // Reached a different component.
                            break;
                        }
                    }
                    (None, Some(_)) => {
                        // Path is a descendant of current node.
                        break;
                    }
                }
            }
            // Go to the closest parent directory.
            self.indices.truncate(num_shared_components + 1);
            // Return the number of strict ancestors between the path and the
            // current directory (0 if direct child).
            path_components.count()
        };

        let num_nodes = self.nodes.len();
        {
            // Add a new child to the parent directory.
            let prev_node = &mut self.nodes[*self.indices.last().unwrap()];
            prev_node.children.push(num_nodes);
        }

        let path_index = num_nodes + extra_depth;

        if extra_depth != 0 {
            // Path is not a direct child of the previous entry, we need to
            // add intermediate parents.
            let mut extra_nodes = Vec::with_capacity(extra_depth);
            let ancestors = path.ancestors().skip(1).take(extra_depth);
            for (i, ancestor) in ancestors.enumerate() {
                extra_nodes.push(TreeNode::new(ancestor, vec![path_index - i]));
            }
            extra_nodes.reverse();
            self.nodes.append(&mut extra_nodes);
            for i in 0..extra_depth {
                self.indices.push(num_nodes + i);
            }
        }

        let path_node = TreeNode::new(path, vec![]);
        self.nodes.push(path_node);
        self.indices.push(path_index);
    }
}
