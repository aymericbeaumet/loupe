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
  pub foobar: u32,
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
      arena: Arena::new(65536),
      root: std::ptr::null_mut(),
    };
    index.root = index.arena.alloc();
    index
  }

  pub fn add_record_slice(&mut self, bytes: &[u8]) {
    let bytes = bytes; // TODO: store in shared memory
    let record: Record = serde_json::from_slice(bytes).unwrap();
    for value in record.attributes.values() {
      if let serde_json::Value::String(value) = value {
        self.add_node(value);
      }
    }
  }

  pub fn add_node(&mut self, key: &str) {
    let root = self.get_root_mut();
    for word in key.unicode_words() {
      println!("{}", word);
    }
  }

  pub fn query(&self, query: &str) -> u32 {
    self.get_root().foobar
  }

  fn get_root(&self) -> &Node256 {
    unsafe { &*self.root }
  }

  fn get_root_mut(&mut self) -> &mut Node256 {
    unsafe { &mut *self.root }
  }
}
