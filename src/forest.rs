use std::collections::{BTreeMap, BTreeSet};

use crate::repo::{Node, Tree};
use crate::upath::UPath;

pub struct Forest {
    open: BTreeMap<UPath, Tree>,
    closed: BTreeSet<UPath>,
}

impl Forest {
    pub fn new() -> Self {
        Self {
            open: BTreeMap::new(),
            closed: BTreeSet::new(),
        }
    }

    pub fn add_node(&mut self, absolute_tree: &UPath, node: Node) {
        let tree = self
            .open
            .entry(absolute_tree.clone())
            .or_insert_with(|| Tree {
                nodes: BTreeSet::new(),
            });

        tree.nodes.insert(node);
    }

    pub fn finish_tree(&mut self, absolute_tree: &UPath) -> Tree {
        let (absolute_tree, tree) = self.open.remove_entry(&absolute_tree).unwrap();
        if self.closed.insert(absolute_tree) {
            panic!()
        }

        tree
    }

    pub fn finish(&self) {
        if !self.open.is_empty() {
            panic!()
        }
    }
}
