use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
  pub id: u128,
  #[serde(flatten)]
  attributes: HashMap<String, serde_json::Value>,
}

impl ToString for Record {
  fn to_string(&self) -> String {
    serde_json::to_string(self).unwrap()
  }
}

impl Record {
  pub fn values(&self) -> impl Iterator<Item = &serde_json::Value> {
    self.attributes.values()
  }
}
