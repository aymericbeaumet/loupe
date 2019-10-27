use crate::arena::Arena;
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

pub struct Node256 {
  children: [*mut Node256; 256],
}

pub struct Index {
  arena: Arena<Node256>,
  root: *mut Node256,
}

unsafe impl Send for Index {}
unsafe impl Sync for Index {}

impl Index {
  pub fn new() -> Self {
    let mut index = Self {
      arena: Arena::new(1_000_000),
      root: std::ptr::null_mut(),
    };
    index.root = index.arena.alloc();
    index
  }

  pub fn add_record_slice(&mut self, bytes: &[u8]) {
    let record: Record = serde_json::from_slice(bytes).unwrap();
    for value in record.attributes.values() {
      if let serde_json::Value::String(value) = value {
        self.insert(value);
      }
    }
  }

  pub fn insert(&mut self, key: &str) {
    let mut current_ptr = self.root;
    for w in key.unicode_words() {
      for c in w.chars() {
        for &b in (c as u32).to_be_bytes().iter() {
          let child_ptr = self.get_node_mut(current_ptr).children[b as usize];
          if !child_ptr.is_null() {
            current_ptr = child_ptr;
          } else {
            let child_ptr = self.arena.alloc();
            self.get_node_mut(current_ptr).children[b as usize] = child_ptr;
            current_ptr = child_ptr;
          }
        }
      }
    }
  }

  pub fn nodes(&self) -> Nodes {
    Nodes {
      index: self,
      stack: vec![self.root],
    }
  }

  pub fn query(&self, _query: &str) -> u32 {
    0
  }

  fn get_node<T>(&self, ptr: *const T) -> &T {
    unsafe { &*ptr }
  }

  fn get_node_mut<T>(&mut self, ptr: *mut T) -> &mut T {
    unsafe { &mut *ptr }
  }
}

pub struct Nodes<'a> {
  index: &'a Index,
  stack: Vec<*mut Node256>,
}

impl<'a> Iterator for Nodes<'a> {
  type Item = &'a Node256;

  fn next(&mut self) -> Option<&'a Node256> {
    let current_ptr = self.stack.pop()?;
    let current_node = self.index.get_node(current_ptr);
    self.stack.extend(
      current_node
        .children
        .iter()
        .filter(|&child| !child.is_null())
        .rev(),
    );
    Some(current_node)
  }
}
