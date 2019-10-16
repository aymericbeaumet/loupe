mod mem;

use mem::graph::Graph;
use nix::unistd::{fork, ForkResult};

fn main() {
    let mut graph = Graph::new();

    match fork() {
        Ok(ForkResult::Parent { .. }) => parent(&mut graph),
        Ok(ForkResult::Child) => child(&graph),
        Err(_) => println!("Fork failed"),
    }
}

fn parent(graph: &mut Graph) {
    let mut node = graph.root_mut();
    node.index = 0xAB0C_D0EF;
}

fn child(graph: &Graph) {
    let node = graph.root();
    println!("{:X?}", node);
}
