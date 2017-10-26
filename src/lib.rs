use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Heap {
    n: i32,
    rank: usize,
    trees: usize,
    marks: i32,
    min: Option<Rc<Node>>,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            n: 0,
            rank: 0,
            trees: 0,
            marks: 0,
            min: None,
        }
    }

    pub fn insert(&mut self, key: i32) -> Weak<Node> {
        self.n += 1;
        self.trees += 1;

        let tmp_node = Node::new(key);
        let ret_node = Rc::downgrade(&tmp_node);

        self.min = if let Some(ref min) = self.min {
            Node::concatenate(Rc::clone(min), Rc::clone(&tmp_node));

            if *tmp_node.key.borrow() < *min.key.borrow() {
                Some(tmp_node)
            } else {
                Some(Rc::clone(min))
            }
        } else {
            Some(tmp_node)
        };

        return ret_node;
    }

    pub fn delete_min(&mut self) {
        self.min = if let Some(ref min) = self.min {
            let min = Rc::clone(min);
            self.n -= 1;
            self.trees -= 1;
            self.trees += *min.rank.borrow();

            let left = match *min.left.borrow() {
                Some(ref left) => Rc::clone(left),
                None => unreachable!(),
            };

            let child = match *min.child.borrow() {
                Some(ref child) => Some(Rc::clone(child)),
                None => None,
            };

            Node::cleanup(Rc::clone(&min));

            if let Some(child) = child {
                if !Rc::ptr_eq(&left, &min) {
                    Node::concatenate(left, Rc::clone(&child));
                }

                Some(Node::find_min(child))
            } else {
                if !Rc::ptr_eq(&left, &min) {
                    Some(Node::find_min(left))
                } else {
                    None
                }
            }
        } else {
            None
        };

        let min = if let Some(ref min) = self.min {
            Some(Rc::clone(min))
        } else {
            None
        };

        if let Some(min) = min {
            let mut nc = NodeConsolidator::new(self.trees);
            nc.consolidate(min);
            self.trees = nc.trees;
            self.rank = nc.rank;
        }
    }

    pub fn print(&self) {
        if let Some(ref min) = self.min {
            let rc = Rc::clone(min);

            for node in NodeIterator::new(rc) {
                node.print(0);
            }
        }
    }

    pub fn reduce_key(&mut self, node: Rc<Node>, key: i32) {
        if key < *node.key.borrow() {
            node.set_key(key);

            self.prune(node);
        }
    }

    fn prune(&mut self, node: Rc<Node>) {
        println!("Called prune on {}", *node.key.borrow());
        let parent = if let Some(ref parent) = *node.parent.borrow() {
            Weak::upgrade(parent)
        } else {
            None
        };

        if let Some(parent) = parent {
            println!("parent: {}", *parent.key.borrow());
            if *parent.key.borrow() < *node.key.borrow() {
                return;
            }

            let child = if let Some(ref child) = *parent.child.borrow() {
                Some(Rc::clone(child))
            } else {
                None
            };

            if let Some(child) = child {
                println!("child exists");
                if Rc::ptr_eq(&child, &node) {
                    println!("node {} is child", *node.key.borrow());
                    if *parent.rank.borrow() > 1 {
                        println!("rank is > 1");
                        if let Some(ref left) = *node.left.borrow() {
                            println!("Setting child to node's left");
                            parent.set_child(Rc::clone(left));
                        }
                    } else {
                        println!("rank <= 1");
                        parent.clear_child();
                    }
                } else {
                    println!("node {} is child", *child.key.borrow());
                }

                let min = if let Some(ref min) = self.min {
                    Some(Rc::clone(min))
                } else {
                    None
                };

                if let Some(min) = min {
                    Node::concatenate(Rc::clone(&min), Node::remove(Rc::clone(&node)));

                    if *node.key.borrow() < *min.key.borrow() {
                        self.min = Some(node);
                    }

                    if parent.is_marked() {
                        self.prune(parent);
                    } else {
                        parent.mark();
                    }
                }
            }
        }
    }
}

pub struct Node {
    key: RefCell<i32>,
    rank: RefCell<usize>,
    marked: RefCell<bool>,
    left: RefCell<Option<Rc<Node>>>,
    right: RefCell<Option<Weak<Node>>>,
    parent: RefCell<Option<Weak<Node>>>,
    child: RefCell<Option<Rc<Node>>>,
}

impl Node {
    fn new(key: i32) -> Rc<Self> {
        let node = Rc::new(Node {
            key: RefCell::new(key),
            rank: RefCell::new(0),
            marked: RefCell::new(false),
            left: RefCell::new(None),
            right: RefCell::new(None),
            parent: RefCell::new(None),
            child: RefCell::new(None),
        });

        node.set_left(Rc::clone(&node));
        node.set_right(Rc::downgrade(&node));

        node
    }

    fn set_key(&self, key: i32) {
        *self.key.borrow_mut() = key;
    }

    fn is_marked(&self) -> bool {
        *self.marked.borrow()
    }

    fn mark(&self) {
        *self.marked.borrow_mut() = true;
    }

    fn increment_rank(&self) {
        *self.rank.borrow_mut() += 1;
    }

    fn decrement_rank(&self) {
        *self.rank.borrow_mut() -= 1;
    }

    fn print(&self, depth: i32) {
        let mut s = String::new();
        for _ in 0..depth {
            s.push_str("  ");
        }
        if self.is_marked() {
            println!("{}{}*:", s, *self.key.borrow());
        } else {
            println!("{}{}:", s, *self.key.borrow());
        }

        if let Some(ref child) = *self.child.borrow() {
            for node in NodeIterator::new(Rc::clone(child)) {
                node.print(depth + 1);
            }
        }
    }

    fn remove(node: Rc<Node>) -> Rc<Node> {
        let left = match *node.left.borrow() {
            Some(ref left) => Rc::clone(left),
            None => unreachable!(),
        };

        let right = match *node.right.borrow() {
            Some(ref right) => Weak::clone(right),
            None => unreachable!(),
        };

        if !Rc::ptr_eq(&node, &left) {
            left.set_right(Weak::clone(&right));

            if let Some(right) = Weak::upgrade(&right) {
                right.set_left(Rc::clone(&left));
            }
        }

        node.set_left(Rc::clone(&node));
        node.set_right(Rc::downgrade(&node));

        node
    }

    fn cleanup(node: Rc<Self>) {
        let n2 = Rc::clone(&node);
        Node::remove(n2);

        let left = match *node.left.borrow() {
            Some(ref left) => Rc::clone(left),
            None => unreachable!(),
        };

        let parent = if let Some(ref parent) = *node.parent.borrow() {
            Weak::upgrade(parent)
        } else {
            None
        }

        if let Some(parent) = parent {
            if let Some(ref child) = *parent.child.borrow() {
                if Rc::ptr_eq(&node, child) {
                    if Rc::ptr_eq(&node, &left) {
                        parent.clear_child();
                    } else {
                        parent.set_child(left);
                    }
                }
            }
        }

        node.clear_left();
        node.clear_right();
        node.clear_parent();
        node.clear_child();
    }

    pub fn set_left(&self, node: Rc<Self>) {
        *self.left.borrow_mut() = Some(node);
    }

    pub fn clear_left(&self) {
        *self.left.borrow_mut() = None;
    }

    pub fn set_right(&self, node: Weak<Self>) {
        *self.right.borrow_mut() = Some(node);
    }

    pub fn clear_right(&self) {
        *self.right.borrow_mut() = None;
    }

    pub fn set_parent(&self, node: Weak<Self>) {
        *self.parent.borrow_mut() = Some(node);
    }

    pub fn clear_parent(&self) {
        *self.parent.borrow_mut() = None;
    }

    pub fn set_child(&self, node: Rc<Self>) {
        *self.child.borrow_mut() = Some(node);
    }

    pub fn clear_child(&self) {
        *self.child.borrow_mut() = None;
    }

    fn concatenate(node1: Rc<Self>, node2: Rc<Self>) {
        let node1_weak = Rc::downgrade(&node1);

        let node2_left = match *node2.left.borrow() {
            Some(ref left) => Rc::clone(left),
            None => unreachable!(),
        };

        let node1_left = match *node1.left.borrow() {
            Some(ref left) => Rc::clone(left),
            None => unreachable!(),
        };

        /*
        println!(
            "Placing {} to {} between {} and {}",
            node2.key,
            node2_left.key,
            node1_left.key,
            node1.key
        );
        */

        let n2parent = if let Some(ref parent) = *node2.parent.borrow() {
            Weak::upgrade(parent)
        } else {
            None
        };

        if let Some(parent) = n2parent {
            if let Some(ref child) = *parent.child.borrow() {
                if Rc::ptr_eq(&node2, child) {
                    parent.clear_child();
                }
            }
        }

        let n1parent = if let Some(ref parent) = *node1.parent.borrow() {
            Some(Weak::clone(parent))
        } else {
            None
        };

        if let Some(parent) = n1parent {
            for node in NodeIterator::new(Rc::clone(&node2)) {
                node.set_parent(Weak::clone(&parent));
            }
        }

        node2_left.set_right(Weak::clone(&node1_weak));
        node1.set_left(Rc::clone(&node2_left));

        node2.set_left(Rc::clone(&node1_left));
        node1_left.set_right(Rc::downgrade(&node2));
    }

    fn find_min(node: Rc<Self>) -> Rc<Self> {
        let mut min_node = Rc::clone(&node);

        for node in NodeIterator::new(node) {
            if *node.key.borrow() < *min_node.key.borrow() {
                min_node = Rc::clone(&node);
            }
        }

        min_node
    }
}

pub struct NodeConsolidator {
    trees: usize,
    rank: usize,
    ranks: HashMap<usize, Rc<Node>>,
}

impl NodeConsolidator {
    fn new(trees: usize) -> Self {
        NodeConsolidator {
            trees: trees,
            rank: 0,
            ranks: HashMap::new(),
        }
    }

    fn consolidate(&mut self, node: Rc<Node>) {
        if self.ranks.len() == self.trees {
            return;
        }

        // println!("Checking {} with rank {}", node.key, *node.rank.borrow());

        let rank = *node.rank.borrow();

        let node2 = Rc::clone(&node);

        let ranked_node = match self.ranks.get(&rank) {
            Some(ref ranked_node) => {
                if !Rc::ptr_eq(&node, ranked_node) {
                    Some(Rc::clone(ranked_node))
                } else {
                    None
                }
            }
            None => None,
        };

        if let Some(ranked_node) = ranked_node {
            let node = if *node.key.borrow() < *ranked_node.key.borrow() {
                self.merge_nodes(node2, ranked_node)
            } else {
                self.merge_nodes(ranked_node, node2)
            };

            self.consolidate(node);
        } else {
            self.ranks.insert(*node.rank.borrow(), Rc::clone(&node));
        }

        let right = if let Some(ref right) = *node.right.borrow() {
            Weak::upgrade(right)
        } else {
            None
        };

        if let Some(right) = right {
            self.consolidate(right);
        }
    }

    fn merge_nodes(&mut self, lesser_node: Rc<Node>, greater_node: Rc<Node>) -> Rc<Node> {
        // println!("Making {} child of {}", greater_node.key, lesser_node.key);
        self.trees -= 1;

        if let Some(ref parent) = *greater_node.parent.borrow() {
            if let Some(parent) = Weak::upgrade(parent) {
                parent.decrement_rank();
            }
        }

        let child = if let Some(ref child) = *lesser_node.child.borrow() {
            Some(Rc::clone(child))
        } else {
            None
        };

        let gn2 = Rc::clone(&greater_node);
        Node::remove(gn2);

        if let Some(child) = child {
            Node::concatenate(child, greater_node);
        } else {
            greater_node.set_parent(Rc::downgrade(&lesser_node));
            lesser_node.set_child(greater_node);
        }

        let rank = *lesser_node.rank.borrow();
        self.ranks.remove(&rank);

        lesser_node.increment_rank();

        if *lesser_node.rank.borrow() > self.rank {
            self.rank = *lesser_node.rank.borrow();
        }

        lesser_node
    }
}

pub struct NodeIterator {
    first: Rc<Node>,
    current: Option<Rc<Node>>,
    first_seen: bool,
}

impl NodeIterator {
    fn new(node: Rc<Node>) -> Self {
        NodeIterator {
            first: Rc::clone(&node),
            current: Some(Rc::clone(&node)),
            first_seen: false,
        }
    }

    fn reset(&mut self) {
        self.current = Some(Rc::clone(&self.first));
        self.first_seen = false;
    }
}

impl Iterator for NodeIterator {
    type Item = Rc<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = match self.current {
            Some(ref current) => Rc::clone(current),
            None => return None,
        };

        if self.first_seen && Rc::ptr_eq(&current, &self.first) {
            return None;
        } else if Rc::ptr_eq(&current, &self.first) {
            self.first_seen = true;
        }

        if let Some(ref right) = *current.right.borrow() {
            self.current = Weak::upgrade(right);
        } else {
            self.current = None;
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
