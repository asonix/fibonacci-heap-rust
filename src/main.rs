extern crate fibonacci_heap;

use std::rc::Weak;
use fibonacci_heap::Heap;

fn main() {
    println!("creating:");
    let mut heap = Heap::new();
    heap.print();
    println!();

    println!("initial:");
    let v: Vec<_> = (1..9).map(|i| heap.insert(i)).collect();

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

    /*

    for _ in 1..17 {
        println!("delete_min:");
        heap.delete_min();
        heap.print();
        println!();
    }

    */
}
