use once_cell::sync::Lazy;
use std::sync::RwLock;

const MIN_DEGREE: usize = 3;
// Minimum keys that a node must have MIN_DEGREE-1 keys
// Maximum keys that a node can have is 2*MIN_DEGREE-1 keys

#[derive(Debug, Clone)]
struct Node {
    keys: Vec<String>,
    values: Vec<String>,
    children: Vec<Option<Box<Node>>>, // recursive allocation
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

    /// Inserts a key-value pair into a node that is not full.
    /// For leaf nodes, it finds the right position (maintaining order) and inserts.
    /// For internal nodes, it finds the correct child and recursively calls insertion.
    ///
    /// Parameters:
    ///
    /// > key: String
    /// > value: String
    fn insert_not_full(&mut self, key: String, val: String) {
        // compare the Node values for insertion ..
        // we need a variable to determine the correct position of insertion ..
        let mut i = self.keys.len();

        // now that you have the length ... you gotta find the position where you have to insert
        // recursively go down on it ...
        if self.is_leaf {
            // incase it is a leaf

            while i > 0 && key < self.keys[i - 1] {
                // keys are in order ... so we just need to keep moving back to find the position
                // in the keys vector ....
                i -= 1;
            }

            // now that we have found the position .. let's just insert them into the current node
            // at the proper position
            self.keys.insert(i, key); // inserts them at the ith index
            self.values.insert(i, val);

            // but what is it is a full leaf node ???
        } else {
            // node is not a leaf
            while i > 0 && key < self.keys[i - 1] {
                i -= 1;
            }

            i += 1; // We want the child right after the key that is less than the new key
            if self.children[i].as_ref().unwrap().keys.len() == 2 * MIN_DEGREE - 1 {
                self.split_child(i);
                // after splitting .. gotta decide which child to descend into
                if key > self.keys[i] {
                    i += 1;
                }
            }

            // recursively insert into the required child --
            self.children[i].as_mut().unwrap().insert_not_full(key, val);
        }
    }

    /// Splits a full child node.
    /// Promotes the median key from the full child to the current node
    ///
    /// Parameters:
    ///
    /// > i: usize (index of the child to split)
    fn split_child(&mut self, i: usize) {
        // y is one of the children -- we gotta split this child
        let mut y = self.children[i]
            .take() // takes the value of the option and leaves a None in its place
            .unwrap();

        let mut z = Box::new(Node::new(y.is_leaf)); // create a new node at the same level as y

        // y gets to keep the first MIN_DEGREE elements and the rest go to z ..
        z.keys = y.keys.split_off(MIN_DEGREE); // splits at the given index! -- split_off is a method of vec![] ...
        z.values = y.values.split_off(MIN_DEGREE);

        if !y.is_leaf {
            // if y is not a leaf .. the children should be split too .. common sense!
            z.children = y.children.split_off(MIN_DEGREE);
            // y will have one less children than what's required ...
        }

        self.children.insert(i + 1, Some(z));
        // remove the last element from y and insert into the parent node at index i
        // median key in y is promoted to parent
        self.keys.insert(i, y.keys.pop().unwrap());
        self.values.insert(i, y.values.pop().unwrap());

        // set back the child node to the newly modified node
        self.children[i] = Some(y);
    }
}

struct Btree {
    root: Option<Box<Node>>,
}

impl Btree {
    /// create a new Btree
    pub fn new() -> Btree {
        Btree { root: None }
    }

    /// Inserts a key-value pair into the BTree
    ///
    /// Parameters:
    ///
    /// > key: String
    /// > value: String
    pub fn insert(&mut self, key: String, value: String) {
        // if there is an existing root
        if let Some(ref mut root) = self.root {
            // if self.root is Some(none) then return a mutable
            // reference to that node

            if root.keys.len() == (MIN_DEGREE * 2 - 1) {
                // if the root is full -- we gotta increase the height
                let mut new_root = Box::new(Node::new(false));
                // clone the current root and make it as the first child of the new root .. but why
                // so?
                new_root.children.push(Some(root.clone()));
                new_root.split_child(0);
                new_root.insert_not_full(key, value);
                self.root = Some(new_root);
            } else {
                root.insert_not_full(key, value);
            }
        } else {
            // simply insert right? .. explain here too!
            let mut root = Box::new(Node::new(true));
            root.keys.push(key);
            root.values.push(value);
            self.root = Some(root);
        }
    }
}

// Okay .. so I understood and implemented this much .. but the deletion part is a huge pain in the
// ass .. so I'll complete my own implementation sometime else ...
