extern crate fibonacci_heap;

use std::rc::Weak;
use fibonacci_heap::{Heap, Node};

fn heap_stuff() -> Vec<Weak<Node<i32>>> {
    println!("creating:");
    let mut heap = Heap::new();
    heap.print();
    println!();

    println!("initial:");
    let v: Vec<_> = (1..17).map(|i| heap.insert(i, i)).collect();

    heap.print();
    println!();

    println!("delete_min:");
    heap.delete_min();
    heap.print();
    println!();

    if let Some(n) = Weak::upgrade(&v[3]) {
        println!("reduce key:");
        heap.reduce_key(n, 1);
        heap.print();
        println!();
    }

    println!("delete_min:");
    heap.delete_min();
    heap.print();
    println!();

    if let Some(n) = Weak::upgrade(&v[7]) {
        println!("delete {}:", n.get_key());
        heap.delete(n);
        heap.print();
        println!();
    }

    v
}

fn main() {
    let v = heap_stuff();

    for (i, node) in v.iter().enumerate() {
        if let Some(node) = Weak::upgrade(&node) {
            println!("Failed to delete {}, index: {}", node.get_key(), i);
        }
    }

}
