use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
  #[serde(default = "default_id")]
  id: u128,
  #[serde(flatten)]
  attributes: HashMap<String, serde_json::Value>,
}

impl ToString for Record {
  fn to_string(&self) -> String {
    serde_json::to_string(self).unwrap()
  }
}

impl Record {
  pub fn id(&self) -> u128 {
    self.id
  }

  pub fn values(&self) -> impl Iterator<Item = &serde_json::Value> {
    self.attributes.values()
  }
}

fn default_id() -> u128 {
  uuid::Uuid::new_v4().as_u128()
}
