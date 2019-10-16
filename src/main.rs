use memmap::{Mmap, MmapMut, MmapOptions};
use nix::unistd::{fork, ForkResult};

#[derive(Debug)]
struct Node {
    padding: u8,
    index: u32,
}

impl Node {
    fn node_ref(mmap: &Mmap, index: isize) -> &Node {
        let offset = index * std::mem::size_of::<Node>() as isize;
        let mut ptr = mmap.as_ptr();
        let mut ptr = unsafe { ptr.offset(offset) } as *const _ as *const Node;
        unsafe { &*ptr }
    }

    fn node_mut_ref(mmap: &mut MmapMut, index: isize) -> &mut Node {
        let offset = index * std::mem::size_of::<Node>() as isize;
        let mut ptr = mmap.as_mut_ptr();
        let mut ptr = unsafe { ptr.offset(offset) } as *mut _ as *mut Node;
        unsafe { &mut *ptr }
    }
}

fn main() {
    let mut mmap = MmapOptions::new()
        .len(1024 * std::mem::size_of::<Node>())
        .map_anon()
        .unwrap();

    match fork() {
        Ok(ForkResult::Parent { .. }) => parent(&mut mmap),
        Ok(ForkResult::Child) => child(&mmap.make_read_only().unwrap()),
        Err(_) => println!("Fork failed"),
    }
}

fn parent(mmap: &mut MmapMut) {
    let node = Node::node_mut_ref(mmap, 10);
    node.padding = 0x2A;
    node.index = 0xAB0C_D0EF;
}

fn child(mmap: &Mmap) {
    let node = Node::node_ref(mmap, 10);
    println!("{:X?}", node);
}
