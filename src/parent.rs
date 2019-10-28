use crate::index::{Index, Record};
use byteorder::{BigEndian, ReadBytesExt};
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
  let index = index.lock().unwrap();
  let mut output = vec!["digraph index {".to_owned()];
  for ((parent_path, _), (child_path, _)) in index.edges() {
    output.push(format!(
      "  \"{}\" -> \"{}\"",
      format_path(&parent_path),
      format_path(&child_path),
    ));
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
  let chunks = path.chunks_exact(std::mem::size_of::<u32>());
  let remainder = chunks.remainder();
  let chars = chunks
    .map(|mut chunk| std::char::from_u32(chunk.read_u32::<BigEndian>().unwrap()).unwrap())
    .collect::<String>();
  if remainder.is_empty() {
    chars
  } else {
    format!("{}{:02X?}", chars, remainder)
  }
}
