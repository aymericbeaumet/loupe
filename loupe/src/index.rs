use crate::arena::{Arena, TypedArena};
use crate::record::Record;
use crate::tokenizer::TokenizerExt;
use itertools::Itertools;
use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Clone, Copy)]
pub struct Node256 {
  children_ptrs: [*mut Node256; 256],
  leaf_ptr: *mut Leaf256,
}

impl Node256 {
  // Find a child of the current node
  pub fn child(&self, key: u8) -> Option<&Self> {
    let child_ptr = self.children_ptrs[key as usize];
    unsafe { child_ptr.as_ref() }
  }

  // Find a child starting from the current node
  pub fn child_deep(&self, keys: &[u8]) -> Option<&Self> {
    keys.iter().try_fold(self, |node, &key| node.child(key))
  }

  // Return an iterator for the children of the current node
  pub fn children(&self) -> impl Iterator<Item = (u8, &Self)> {
    self
      .children_ptrs
      .iter()
      .enumerate()
      .filter_map(|(key, child_ptr)| {
        unsafe { child_ptr.as_ref() }.map(|child_node| (key as u8, child_node))
      })
  }

  // Return an iterator for all the children starting from the current node
  pub fn children_deep(&self) -> Box<dyn Iterator<Item = (u8, &Self)> + '_> {
    Box::new(
      self
        .children()
        .flat_map(|(key, child)| std::iter::once((key, child)).chain(child.children_deep())),
    )
  }

  // Return an iterator for the records of the current node
  pub fn records(&self) -> impl Iterator<Item = Record> {
    unsafe { self.leaf_ptr.as_ref() }
      .into_iter()
      .flat_map(|leaf| leaf.records.chunks_exact(2))
      .map(|chunk| (chunk[0], chunk[1]))
      .take_while(|(ptr_start, ptr_end)| !ptr_start.is_null() && !ptr_end.is_null())
      .map(|(ptr_start, ptr_end)| {
        let len = ptr_end as usize - ptr_start as usize;
        let bytes = unsafe { std::slice::from_raw_parts(ptr_start, len) };
        serde_json::from_slice(bytes).unwrap()
      })
  }

  // Return an iterator for the all the records starting from the current node
  pub fn records_deep(&self) -> impl Iterator<Item = Record> + '_ {
    self.records().chain(
      self
        .children_deep()
        .flat_map(|(_key, child)| child.records()),
    )
  }
}

impl Serialize for Node256 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("Node256", 2)?;
    state.serialize_field("children", &self.children().collect::<Vec<_>>())?;
    state.serialize_field("records", &self.records().collect::<Vec<_>>())?;
    state.end()
  }
}

pub struct Leaf256 {
  records: [*const u8; 2 * 128],
}

pub struct Index {
  root_ptr: *mut Node256,
  nodes: TypedArena<Node256>,
  leaves: TypedArena<Leaf256>,
  records: Arena,
}

unsafe impl Send for Index {}
unsafe impl Sync for Index {}

impl Index {
  pub fn new() -> Self {
    let mut index = Self {
      root_ptr: std::ptr::null_mut(),
      nodes: TypedArena::new(100_000_000),
      leaves: TypedArena::new(100_000_000),
      records: Arena::new(100_000_000),
    };
    index.root_ptr = index.nodes.alloc().unwrap();
    index
  }

  pub fn root_node(&self) -> &Node256 {
    unsafe { &*self.root_ptr }
  }

  pub fn add_record_slice(&mut self, bytes: &[u8]) {
    let stored = self.records.store(bytes).unwrap();
    let record: Record = serde_json::from_slice(bytes).unwrap();
    record.values().for_each(|value| {
      if let serde_json::Value::String(value) = value {
        self.insert(value, stored);
      }
    })
  }

  pub fn insert(&mut self, key: &str, (record_ptr_start, record_ptr_end): (*const u8, *const u8)) {
    key.tokenize().for_each(|token| {
      let mut current_ptr = self.root_ptr;
      token.bytes().for_each(|byte| {
        let current_node = unsafe { &mut *current_ptr };
        let child_ptr = current_node.children_ptrs[byte as usize];
        if !child_ptr.is_null() {
          current_ptr = child_ptr;
        } else {
          let child_ptr = self.nodes.alloc().unwrap();
          let child_node = unsafe { &mut *current_ptr };
          child_node.children_ptrs[byte as usize] = child_ptr;
          current_ptr = child_ptr;
        }
      });
      let mut current_node = unsafe { &mut *current_ptr };
      if current_node.leaf_ptr.is_null() {
        current_node.leaf_ptr = self.leaves.alloc().unwrap();
      }
      let leaf = unsafe { &mut *current_node.leaf_ptr };
      for i in (0..leaf.records.len()).step_by(2) {
        if leaf.records[i] == record_ptr_start {
          break;
        }
        if leaf.records[i].is_null() {
          leaf.records[i] = record_ptr_start;
          leaf.records[i + 1] = record_ptr_end;
          break;
        }
      }
    });
  }

  pub fn query_nodes<'a>(
    &'a self,
    query: &'a str,
  ) -> impl Iterator<Item = (String, &Node256)> + 'a {
    let root_node = self.root_node();
    query.tokenize().filter_map(move |token| {
      root_node
        .child_deep(token.as_bytes())
        .map(|child| (token, child))
    })
  }

  pub fn query_records<'a>(&'a self, query: &'a str) -> impl Iterator<Item = Record> + 'a {
    self
      .query_nodes(query)
      .flat_map(|(_word, node)| node.records_deep())
      .unique_by(|record| record.id)
  }
}
