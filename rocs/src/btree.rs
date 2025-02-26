use once_cell::sync::Lazy;
use std::sync::RwLock;

const min_d: usize = 3; // minimum degree of the B-Tree

#[derive(Clone)]
struct Node {
    keys: Vec<String>,
    values: Vec<String>,
    children: Vec<Node>,
    is_leaf: bool,
}

impl Node {
    /// defines a new BTreeNode
    ///
    /// Parameters:
    ///
    /// > leaf: bool
    fn new(leaf: bool) -> Self {
        Node {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            leaf,
        }
    }
}
