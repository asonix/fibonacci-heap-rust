extern crate fibonacci_heap;

use fibonacci_heap::Heap;

fn main() {
    println!("creating:");
    let mut heap = Heap::new();
    heap.print();
    println!();

    println!("initial:");
    for i in 1..17 {
        heap.insert(i);
    }
    heap.print();
    println!();

    for _ in 1..17 {
        println!("delete_min:");
        heap.delete_min();
        heap.print();
        println!();
    }
}
