use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

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

pub struct Index {
  store: Store,
}

impl Index {
  pub fn new() -> Self {
    Self {
      store: Store::new(4096),
    }
  }

  pub fn add_record_slice(&mut self, bytes: &[u8]) {
    let slice = self.store.append(bytes);
    let record: Record = serde_json::from_slice(slice).unwrap();
    println!("{:#?}", record);
  }
}
