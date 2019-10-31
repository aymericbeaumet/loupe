use crate::index::Index;
use crate::index::Node256;
use crate::record::Record;
use rocket::config::{Config, Environment};
use rocket::State;
use rocket_contrib::json::Json;
use rocksdb::{WriteBatch, DB};
use std::sync::{Arc, Mutex};

#[post("/records", format = "json", data = "<records>")]
fn add_records(
  db: State<DB>,
  index: State<Arc<Mutex<Index>>>,
  records: Json<Vec<Record>>,
) -> &'static str {
  let mut batch = WriteBatch::default();
  for record in records.into_inner() {
    let id_as_bytes = record.id.to_ne_bytes();
    let record_as_string = record.to_string();
    let record_as_bytes = record_as_string.as_bytes();
    batch.put(id_as_bytes, record_as_bytes).unwrap();
    {
      let mut index = index.lock().unwrap();
      index.add_record_slice(record_as_bytes);
    }
  }
  db.write(batch).unwrap();
  // TODO: add record to the index as batch after they have been written to disk
  ""
}

#[get("/debug/nodes")]
fn nodes(index: State<Arc<Mutex<Index>>>) -> Json<Node256> {
  let index = index.lock().unwrap();
  Json(*index.root_node())
}

pub fn main(index: Index) -> Result<(), Box<dyn std::error::Error>> {
  let index = Arc::new(Mutex::new(index));
  let db = DB::open_default("records.rocksdb")?;

  {
    let mut index = index.lock().unwrap();
    for (_, record) in db.iterator(rocksdb::IteratorMode::Start) {
      index.add_record_slice(&record);
    }
  }

  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(9191)
    .finalize()?;
  rocket::custom(config)
    .manage(db)
    .manage(index)
    .mount("/", routes![add_records, nodes])
    .launch();

  Ok(())
}
