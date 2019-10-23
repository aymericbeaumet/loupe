use crate::index::Index;
use rocket::config::{Config, Environment};
use rocket_contrib::json::Json;
use rocksdb::DB;
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
struct Record {
  #[serde(default = "default_id")]
  id: String,
  #[serde(default)]
  tags: Vec<String>,
  #[serde(default = "default_now")]
  created_at: u64,
  #[serde(default = "default_now")]
  updated_at: u64,

  #[serde(flatten)]
  attributes: HashMap<String, serde_json::Value>,
}

#[post("/records", format = "json", data = "<records>")]
fn index(records: Json<Vec<Record>>) -> &'static str {
  println!("{:#?}", records);
  "Hello, parent!"
}

pub fn main(index: &mut Index) -> Result<(), Box<dyn std::error::Error>> {
  let db = DB::open_default("records.rocksdb")?;
  db.put(b"my key", b"my value")?;
  match db.get(b"my key") {
    Ok(Some(value)) => println!("retrieved value {}", value.to_utf8().unwrap()),
    Ok(None) => println!("value not found"),
    Err(e) => println!("operational problem encountered: {}", e),
  }
  db.delete(b"my key")?;

  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(9191)
    .finalize()?;
  rocket::custom(config).mount("/", routes![index]).launch();

  Ok(())
}
