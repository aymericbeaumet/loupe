use crate::index::Index;
use rocket::config::{Config, Environment};
use rocket::State;
use rocket_contrib::json::Json;
use rocksdb::{WriteBatch, DB};
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
fn add_records(db: State<DB>, records: Json<Vec<Record>>) -> &'static str {
  let mut batch = WriteBatch::default();
  for record in records.into_inner() {
    batch
      .put(
        record.id.as_bytes(),
        serde_json::to_string(&record).unwrap(),
      )
      .unwrap();
  }
  db.write(batch).unwrap();
  ""
}

pub fn main(index: &mut Index) -> Result<(), Box<dyn std::error::Error>> {
  let db = DB::open_default("records.rocksdb")?;

  for (_, value) in db.iterator(rocksdb::IteratorMode::Start) {
    let value: Record = serde_json::from_slice(&value).unwrap();
    println!("{:#?}", value);
  }

  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(9191)
    .finalize()?;
  rocket::custom(config)
    .manage(db)
    .mount("/", routes![add_records])
    .launch();

  Ok(())
}
