use std::collections::HashMap;
use std::cell::{Ref, RefCell};
use std::rc::{Rc, Weak};

pub struct Heap<T> {
    n: i32,
    rank: usize,
    trees: usize,
    marks: i32,
    min: Option<Rc<Node<T>>>,
}

impl<T> Heap<T> {
    pub fn new() -> Self {
        Heap {
            n: 0,
            rank: 0,
            trees: 0,
            marks: 0,
            min: None,
        }
    }

    fn get_min(&self) -> Option<Rc<Node<T>>> {
        Node::from_borrowed_option_rc(&self.min)
    }

    pub fn insert(&mut self, key: i32, value: T) -> Weak<Node<T>> {
        self.n += 1;
        self.trees += 1;

        let tmp_node = Node::new(key, value);
        let ret_node = Rc::downgrade(&tmp_node);

        self.min = if let Some(ref min) = self.min {
            Node::concatenate(Rc::clone(min), Rc::clone(&tmp_node));

            if tmp_node.get_key() < min.get_key() {
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
        self.min = if let Some(min) = self.get_min() {
            self.n -= 1;
            self.trees -= 1;
            self.trees += min.get_rank();

            let left = match min.get_left() {
                Some(left) => left,
                None => unreachable!(),
            };

            let child = min.get_child();

            Node::remove(Rc::clone(&min));
            min.clear_child();
            min.clear_left();

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

        if let Some(min) = self.get_min() {
            let mut nc = NodeConsolidator::new(self.trees);
            nc.consolidate(min);
            self.trees = nc.trees;
            self.rank = nc.rank;
        }
    }

    pub fn print(&self) {
        if let Some(min) = self.get_min() {
            for node in NodeIterator::new(min) {
                node.print(0);
            }
        }
    }

    pub fn reduce_key(&mut self, node: Rc<Node<T>>, key: i32) {
        if key < node.get_key() {
            node.set_key(key);

            self.prune(node);
        }
    }

    pub fn delete(&mut self, node: Rc<Node<T>>) {
        self.reduce_key(node, i32::min_value());
        self.delete_min();
    }

    pub fn union(&mut self, mut heap: Heap<T>) {
        if let Some(f_min) = heap.get_min() {
            if let Some(l_min) = self.get_min() {
                if f_min.get_key() < l_min.get_key() {
                    self.min = Some(Rc::clone(&f_min));
                }

                Node::concatenate(l_min, f_min);
            } else {
                self.min = Some(f_min);
            }
            heap.min = None
        }
    }

    fn prune(&mut self, node: Rc<Node<T>>) {
        if let Some(parent) = node.get_parent() {
            if parent.get_key() < node.get_key() {
                return;
            }

            if node.is_marked() {
                node.unmark();
                self.marks -= 1;
            }

            if let Some(child) = parent.get_child() {
                if Rc::ptr_eq(&child, &node) {
                    if parent.get_rank() > 1 {
                        if let Some(left) = node.get_left() {
                            parent.set_child(left);
                        }
                    } else {
                        parent.clear_child();
                    }
                }

                parent.decrement_rank();

                if let Some(min) = self.get_min() {
                    Node::concatenate(Rc::clone(&min), Node::remove(Rc::clone(&node)));
                    self.trees += 1;

                    if node.get_key() < min.get_key() {
                        self.min = Some(node);
                    }

                    if parent.is_marked() {
                        self.prune(parent);
                    } else {
                        self.marks += 1;
                        parent.mark();
                    }
                }
            }
        }
    }
}

impl<T> Drop for Heap<T> {
    fn drop(&mut self) {
        if let Some(min) = self.get_min() {
            for node in NodeIterator::new(min) {
                Node::cleanup(node);
            }
        }
    }
}

pub struct Node<T> {
    key: RefCell<i32>,
    value: T,
    rank: RefCell<usize>,
    marked: RefCell<bool>,
    left: RefCell<Option<Rc<Node<T>>>>,
    right: RefCell<Option<Weak<Node<T>>>>,
    parent: RefCell<Option<Weak<Node<T>>>>,
    child: RefCell<Option<Rc<Node<T>>>>,
}

impl<T> Node<T> {
    fn new(key: i32, value: T) -> Rc<Self> {
        let node = Rc::new(Node {
            key: RefCell::new(key),
            value: value,
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

    pub fn get_value(self) -> T {
        self.value
    }

    pub fn get_key(&self) -> i32 {
        *self.key.borrow()
    }

    fn set_key(&self, key: i32) {
        *self.key.borrow_mut() = key;
    }

    fn get_rank(&self) -> usize {
        *self.rank.borrow()
    }

    fn is_marked(&self) -> bool {
        *self.marked.borrow()
    }

    fn mark(&self) {
        *self.marked.borrow_mut() = true;
    }

    fn unmark(&self) {
        *self.marked.borrow_mut() = false;
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
            println!("{}{}*:", s, self.get_key());
        } else {
            println!("{}{}:", s, self.get_key());
        }

        if let Some(child) = self.get_child() {
            for node in NodeIterator::new(child) {
                node.print(depth + 1);
            }
        }
    }

    fn remove(node: Rc<Node<T>>) -> Rc<Node<T>> {
        let left = match node.get_left() {
            Some(left) => left,
            None => unreachable!(),
        };

        let right = match node.get_right() {
            Some(right) => right,
            None => unreachable!(),
        };

        if !Rc::ptr_eq(&node, &left) {
            left.set_right(Rc::downgrade(&right));
            right.set_left(Rc::clone(&left));
        }

        if let Some(parent) = node.get_parent() {
            if let Some(child) = parent.get_child() {
                if Rc::ptr_eq(&node, &child) {
                    if Rc::ptr_eq(&node, &left) {
                        parent.clear_child();
                    } else {
                        parent.set_child(left);
                    }
                }
            }
        }

        node.set_left(Rc::clone(&node));
        node.set_right(Rc::downgrade(&node));

        node
    }

    fn cleanup(node: Rc<Self>) {
        if let Some(child) = node.get_child() {
            for node in NodeIterator::new(child) {
                Node::cleanup(node);
            }
        }

        node.clear_left();
        node.clear_child();
    }

    fn set_left(&self, node: Rc<Self>) {
        *self.left.borrow_mut() = Some(node);
    }

    fn get_left(&self) -> Option<Rc<Self>> {
        Node::from_ref_option_rc(self.left.borrow())
    }

    fn clear_left(&self) {
        *self.left.borrow_mut() = None;
    }

    fn set_right(&self, node: Weak<Self>) {
        *self.right.borrow_mut() = Some(node);
    }

    fn get_right(&self) -> Option<Rc<Self>> {
        Node::from_ref_option_weak(self.right.borrow())
    }

    fn set_parent(&self, node: Weak<Self>) {
        *self.parent.borrow_mut() = Some(node);
    }

    fn get_parent(&self) -> Option<Rc<Self>> {
        Node::from_ref_option_weak(self.parent.borrow())
    }

    fn set_child(&self, node: Rc<Self>) {
        *self.child.borrow_mut() = Some(node);
    }

    fn get_child(&self) -> Option<Rc<Self>> {
        Node::from_ref_option_rc(self.child.borrow())
    }

    fn clear_child(&self) {
        *self.child.borrow_mut() = None;
    }

    fn from_ref_option_rc(node: Ref<Option<Rc<Self>>>) -> Option<Rc<Self>> {
        if let Some(ref n) = *node {
            Some(Rc::clone(n))
        } else {
            None
        }
    }

    fn from_borrowed_option_rc(node: &Option<Rc<Self>>) -> Option<Rc<Self>> {
        if let &Some(ref n) = node {
            Some(Rc::clone(n))
        } else {
            None
        }
    }

    fn from_ref_option_weak(node: Ref<Option<Weak<Self>>>) -> Option<Rc<Self>> {
        if let Some(ref n) = *node {
            Weak::upgrade(n)
        } else {
            None
        }
    }

    fn concatenate(node1: Rc<Self>, node2: Rc<Self>) {
        let node1_weak = Rc::downgrade(&node1);

        let node2_left = match node2.get_left() {
            Some(left) => left,
            None => unreachable!(),
        };

        let node1_left = match node1.get_left() {
            Some(left) => left,
            None => unreachable!(),
        };

        if let Some(parent) = node2.get_parent() {
            if let Some(child) = parent.get_child() {
                if Rc::ptr_eq(&node2, &child) {
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
            if node.get_key() < min_node.get_key() {
                min_node = Rc::clone(&node);
            }
        }

        min_node
    }
}

pub struct NodeConsolidator<T> {
    trees: usize,
    rank: usize,
    ranks: HashMap<usize, Rc<Node<T>>>,
}

impl<T> NodeConsolidator<T> {
    fn new(trees: usize) -> Self {
        NodeConsolidator {
            trees: trees,
            rank: 0,
            ranks: HashMap::new(),
        }
    }

    fn consolidate(&mut self, node: Rc<Node<T>>) {
        if self.ranks.len() == self.trees {
            return;
        }

        let rank = node.get_rank();

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
            let node = if node.get_key() < ranked_node.get_key() {
                self.merge_nodes(node2, ranked_node)
            } else {
                self.merge_nodes(ranked_node, node2)
            };

            self.consolidate(node);
        } else {
            self.ranks.insert(node.get_rank(), Rc::clone(&node));
        }

        if let Some(right) = node.get_right() {
            self.consolidate(right);
        }
    }

    fn merge_nodes(&mut self, lesser_node: Rc<Node<T>>, greater_node: Rc<Node<T>>) -> Rc<Node<T>> {
        self.trees -= 1;

        if let Some(parent) = greater_node.get_parent() {
            parent.decrement_rank();
        }

        let gn2 = Rc::clone(&greater_node);
        Node::remove(gn2);

        if let Some(child) = lesser_node.get_child() {
            Node::concatenate(child, greater_node);
        } else {
            greater_node.set_parent(Rc::downgrade(&lesser_node));
            lesser_node.set_child(greater_node);
        }

        let rank = lesser_node.get_rank();
        self.ranks.remove(&rank);

        lesser_node.increment_rank();

        if lesser_node.get_rank() > self.rank {
            self.rank = lesser_node.get_rank();
        }

        lesser_node
    }
}

pub struct NodeIterator<T> {
    first: Rc<Node<T>>,
    current: Option<Rc<Node<T>>>,
    first_seen: bool,
}

impl<T> NodeIterator<T> {
    fn new(node: Rc<Node<T>>) -> Self {
        NodeIterator {
            first: Rc::clone(&node),
            current: Some(Rc::clone(&node)),
            first_seen: false,
        }
    }
}

impl<T> Iterator for NodeIterator<T> {
    type Item = Rc<Node<T>>;

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

        self.current = current.get_left();

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
