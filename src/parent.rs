use crate::index::{Index, Record};
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
    let as_string = serde_json::to_string(&record).unwrap();
    let as_bytes = as_string.as_bytes();
    batch.put(record.id.as_bytes(), as_bytes).unwrap();
    {
      let mut index = index.lock().unwrap();
      index.add_record_slice(as_bytes);
    }
  }
  db.write(batch).unwrap();
  // TODO: add record to the index as batch after they have been written to disk
  ""
}

#[get("/debug/dot", format = "json")]
fn dot(index: State<Arc<Mutex<Index>>>) -> String {
  let mut output = vec!["digraph index {".to_owned()];
  {
    let index = index.lock().unwrap();
    for ((parent_path, _), (child_path, _)) in index.edges() {
      output.push(format!(
        "  \"{}\" -> \"{}\"",
        format_path(&parent_path),
        format_path(&child_path),
      ));
    }
  }
  output.push("}".to_owned());
  output.push("".to_owned());
  output.join("\n")
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
    .mount("/", routes![add_records, dot])
    .launch();

  Ok(())
}

fn format_path(path: &[u8]) -> String {
  match std::str::from_utf8(&path) {
    Ok(s) => s.to_owned(),
    Err(error) => format!(
      "{}{:02X?}",
      format_path(&path[..error.valid_up_to()]),
      &path[error.valid_up_to()..]
    ),
  }
}
