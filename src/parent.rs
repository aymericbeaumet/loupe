use crate::index::{Index, Record};
use rocket::config::{Config, Environment};
use rocket::State;
use rocket_contrib::json::Json;
use rocksdb::{WriteBatch, DB};

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

  for (_, record) in db.iterator(rocksdb::IteratorMode::Start) {
    index.add_record_slice(&record);
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
