use super::store::Store;

#[derive(Debug)]
pub struct Graph {
  store: Store<Node>,
}

impl Graph {
  pub fn new() -> Self {
    Self {
      store: Store::new(1024),
    }
  }

  pub fn node(&self, index: usize) -> &Node {
    self.store.chunk(index)
  }

  pub fn node_mut(&mut self, index: usize) -> &mut Node {
    self.store.chunk_mut(index)
  }

  pub fn root(&self) -> &Node {
    self.node(0)
  }

  pub fn root_mut(&mut self) -> &mut Node {
    self.node_mut(0)
  }
}

#[derive(Debug)]
pub struct Node {
  pub index: u32,
}
