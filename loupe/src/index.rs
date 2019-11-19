use crate::arena::{Arena, ArenaSliceKey, ArenaTypeKey};
use crate::record::Record;
use crate::tokenizer::TokenizerExt;
use itertools::Itertools;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::mem;
use std::sync::atomic::Ordering;

lazy_static! {
  static ref ARENA: Arena = Arena::new();
}

#[derive(Copy, Clone)]
pub struct Node {
  children: [ArenaTypeKey<Node>; 256],
  records: [ArenaSliceKey<u8>; 256],
}

impl Node {
  // Add a child to the current node
  pub fn add_child(&self, key: u8, child: ArenaTypeKey<Self>) {
    let dst = ARENA.atomic(self.children[key as usize]);
    unsafe { dst.store(mem::transmute_copy(&child), Ordering::Release) };
  }

  // Find a child of the current node
  pub fn child(&self, key: u8) -> Option<&Self> {
    ARENA.get(self.children[key as usize])
  }

  // Find a child starting from the current node
  pub fn child_deep(&self, keys: &[u8]) -> Option<&Self> {
    keys.iter().try_fold(self, |node, &key| node.child(key))
  }

  // Return an iterator for the children of the current node
  pub fn children(&self) -> impl Iterator<Item = (u8, &Self)> {
    self
      .children
      .iter()
      .enumerate()
      .filter_map(|(key, child_key)| {
        ARENA
          .get(*child_key)
          .map(|child_node| (key as u8, child_node))
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

  pub fn add_record(&self, key: ArenaSliceKey<u8>) {
    for (i, &record_key) in self.records.iter().enumerate() {
      if record_key == key {
        break;
      }
      let dst = ARENA.atomic_slice(self.records[i]);
      let previous =
        unsafe { dst.compare_and_swap(0, mem::transmute_copy(&key), Ordering::Release) };
      if previous == 0 {
        break;
      }
    }
  }

  // Return an iterator for the records of the current node
  pub fn records(&self) -> impl Iterator<Item = Record> + '_ {
    self
      .records
      .iter()
      .take_while(|key| !key.is_null())
      .map(|key| {
        let bytes = unsafe { ARENA.get_slice_unchecked(key) };
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

impl Serialize for Node {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("Node", 2)?;
    state.serialize_field("children", &self.children().collect::<Vec<_>>())?;
    state.serialize_field("records", &self.records().collect::<Vec<_>>())?;
    state.end()
  }
}

/// Index contains the root to an AST storing both the records and the associated
/// ART nodes. The data is stored in an arena, and cannot be freed for now. On
/// top of this the root_key is immutable, which means the Index can safely and
/// inexpensively be copied/cloned around.
#[derive(Copy, Clone)]
pub struct Index {
  root_key: ArenaTypeKey<Node>, // TODO: store in u8 instead -> ArenaTypeKey<u8, Node>
}

impl Index {
  pub fn new() -> Self {
    let root_key = ARENA.alloc();
    Self { root_key }
  }

  pub fn add_record_slice(self, bytes: &[u8]) {
    let stored = ARENA.alloc_slice_copy(bytes);
    let record: Record = serde_json::from_slice(bytes).unwrap();
    record.values().for_each(|value| {
      if let serde_json::Value::String(value) = value {
        self.insert(value, stored);
      }
    })
  }

  pub fn insert(self, key: &str, record_key: ArenaSliceKey<u8>) {
    key.tokenize().into_iter().for_each(|token| {
      let insertion_node = token.bytes().fold(
        unsafe { ARENA.get_unchecked(self.root_key) },
        |node, byte| {
          debug!("{} {}", token, byte);
          if let Some(child) = node.child(byte) {
            child
          } else {
            let child_key = ARENA.alloc();
            let child = unsafe { ARENA.get_unchecked(child_key) };
            node.add_child(byte, child_key);
            child
          }
        },
      );
      insertion_node.add_record(record_key);
    });
  }

  pub fn query_nodes(self, query: &str) -> impl Iterator<Item = (String, &Node)> {
    let root = unsafe { ARENA.get_unchecked(self.root_key) };
    query.tokenize().into_iter().filter_map(move |token| {
      root
        .child_deep(token.as_bytes())
        .map(|child| (token, child))
    })
  }

  pub fn query_records(self, query: &str) -> impl Iterator<Item = Record> + '_ {
    self
      .query_nodes(query)
      .flat_map(|(_word, node)| node.records_deep())
      .unique_by(|record| record.id)
  }
}
