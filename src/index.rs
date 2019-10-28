use crate::arena::{Arena, TypedArena};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use unicode_segmentation::UnicodeSegmentation;

fn default_id() -> String {
  uuid::Uuid::new_v4().to_string()
}

fn default_now() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_millis() as u64
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
  #[serde(default = "default_id")]
  pub id: String,
  #[serde(default)]
  pub tags: Vec<String>,
  #[serde(default = "default_now")]
  pub created_at: u64,
  #[serde(default = "default_now")]
  pub updated_at: u64,
  #[serde(flatten)]
  pub attributes: HashMap<String, serde_json::Value>,
}

impl ToString for Record {
  fn to_string(&self) -> String {
    serde_json::to_string(self).unwrap()
  }
}

pub struct Node256 {
  children: [*mut Node256; 256],
  leaf: *mut Leaf256,
}

impl Node256 {
  pub fn records(&self) -> impl Iterator<Item = Record> + '_ {
    let chunks = if self.leaf.is_null() {
      [].chunks_exact(2)
    } else {
      (unsafe { &*self.leaf }).records.chunks_exact(2)
    };
    chunks.take_while(|chunk| !chunk[0].is_null()).map(|chunk| {
      let ptr_start = chunk[0];
      let ptr_end = chunk[1];
      let bytes =
        unsafe { std::slice::from_raw_parts(ptr_start, ptr_end as usize - ptr_start as usize) };
      serde_json::from_slice(bytes).unwrap()
    })
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

  pub fn add_record_slice(&mut self, bytes: &[u8]) {
    let stored = self.records.store(bytes).unwrap();
    let record: Record = serde_json::from_slice(bytes).unwrap();
    for value in record.attributes.values() {
      if let serde_json::Value::String(value) = value {
        self.insert(value, stored);
      }
    }
  }

  pub fn insert(&mut self, key: &str, (record_ptr_start, record_ptr_end): (*const u8, *const u8)) {
    for w in key.unicode_words() {
      let mut current_ptr = self.root_ptr;
      for b in w.bytes() {
        let child_ptr = self.nodes.at_mut(current_ptr).children[b as usize];
        if !child_ptr.is_null() {
          current_ptr = child_ptr;
        } else {
          let child_ptr = self.nodes.alloc().unwrap();
          self.nodes.at_mut(current_ptr).children[b as usize] = child_ptr;
          current_ptr = child_ptr;
        }
      }
      let mut current_node = self.nodes.at_mut(current_ptr);
      if current_node.leaf.is_null() {
        current_node.leaf = self.leaves.alloc().unwrap();
      }
      let leaf = self.leaves.at_mut(current_node.leaf);
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
    }
  }

  pub fn nodes(&self) -> Nodes {
    Nodes {
      index: self,
      stack: vec![(vec![], self.root_ptr)],
    }
  }

  pub fn edges(&self) -> impl Iterator<Item = ((Vec<u8>, &Node256), (Vec<u8>, &Node256))> + '_ {
    self.nodes().flat_map(move |(parent_path, parent_ptr)| {
      let parent_node = self.nodes.at(parent_ptr);
      parent_node
        .children
        .iter()
        .enumerate()
        .filter_map(move |(child_key, &child_ptr)| {
          if child_ptr.is_null() {
            None
          } else {
            let mut child_path = parent_path.clone();
            child_path.push(child_key as u8);
            Some((
              (parent_path.clone(), parent_node),
              (child_path, self.nodes.at(child_ptr)),
            ))
          }
        })
    })
  }

  pub fn query(&self, _query: &str) -> u32 {
    0
  }
}

pub struct Nodes<'a> {
  index: &'a Index,
  stack: Vec<(Vec<u8>, *const Node256)>,
}

impl<'a> Iterator for Nodes<'a> {
  type Item = (Vec<u8>, &'a Node256);

  fn next(&mut self) -> Option<(Vec<u8>, &'a Node256)> {
    let (path, current_ptr) = self.stack.pop()?;
    let current_node = self.index.nodes.at(current_ptr);
    self.stack.extend(
      current_node
        .children
        .iter()
        .enumerate()
        .filter_map(|(key, &child_ptr)| {
          if child_ptr.is_null() {
            None
          } else {
            let mut child_path = path.clone();
            child_path.push(key as u8);
            Some((child_path, child_ptr as *const Node256))
          }
        })
        .rev(),
    );
    Some((path, current_node))
  }
}
