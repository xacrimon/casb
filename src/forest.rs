use std::collections::{BTreeMap, BTreeSet};

use slab::Slab;

use crate::repo::{Node, Tree};
use crate::useg::{UPath, USeg};

struct Branch {
    nodes: Option<BTreeSet<Node>>,
    children: BTreeMap<USeg, usize>,
}

pub struct Forest {
    branches: Slab<Branch>,
    root_key: usize,
}

impl Forest {
    pub fn new() -> Self {
        let mut branches = Slab::new();
        let root = Branch {
            nodes: None,
            children: BTreeMap::new(),
        };

        let root_key = branches.insert(root);
        Self { branches, root_key }
    }

    fn resolve(&self, tree: &UPath) -> (usize, usize) {
        let mut segments = tree.segments().iter();
        let first = segments.next().unwrap();
        let mut parent_key = self.root_key;
        let mut key = *self.branches[parent_key].children.get(first).unwrap();
        for segment in segments {
            parent_key = key;
            key = *self.branches[key].children.get(segment).unwrap();
        }

        (parent_key, key)
    }

    pub fn add_node(&mut self, tree: &UPath, node: Node) {
        let (_, key) = self.resolve(tree);
        let branch = self.branches.get_mut(key).unwrap();
        branch.nodes.as_mut().unwrap().insert(node);
    }

    pub fn finish_tree(&mut self, tree: &UPath) -> Tree {
        let (parent_key, key) = self.resolve(tree);
        let parent = self.branches.get_mut(parent_key).unwrap();
        parent.children.remove(tree.segments().last().unwrap());
        let branch = self.branches.get_mut(key).unwrap();
        let nodes = branch.nodes.take().unwrap();
        Tree { nodes }
    }

    pub fn assert_finished(&self) {
        for (_, branch) in self.branches.iter() {
            if !branch.children.is_empty() {
                panic!()
            }
        }
    }
}
