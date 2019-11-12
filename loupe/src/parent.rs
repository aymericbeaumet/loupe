use crate::index::{Index, Node};
use crate::record::Record;
use rocket::config::{Config, Environment};
use rocket::fairing::AdHoc;
use rocket::http::{Header, RawStr, Status};
use rocket::response::Response;
use rocket::State;
use rocket_contrib::json::Json;
use rocksdb::{WriteBatch, DB};
use std::collections::HashMap;

#[post("/records", format = "json", data = "<records>")]
fn add_records(db: State<DB>, index: State<Index>, records: Json<Vec<Record>>) -> &'static str {
  let mut batch = WriteBatch::default();
  for record in records.into_inner() {
    let id_as_bytes = record.id.to_ne_bytes();
    let record_as_string = record.to_string();
    let record_as_bytes = record_as_string.as_bytes();
    batch.put(id_as_bytes, record_as_bytes).unwrap();
    index.add_record_slice(record_as_bytes);
  }
  db.write(batch).unwrap();
  // TODO: add record to the index as batch after they have been written to disk
  ""
}

#[get("/debug/nodes?<query>")]
fn debug_nodes<'a>(
  index: State<&'a Index>,
  query: &RawStr,
) -> Result<Json<HashMap<String, &'a Node>>, Box<dyn std::error::Error>> {
  let nodes = index.query_nodes(&query.url_decode()?).collect();
  Ok(Json(nodes))
}

#[options("/debug/nodes")]
fn debug_nodes_cors() -> Result<Response<'static>, Status> {
  Response::build()
    .status(Status::NoContent)
    .raw_header("Access-Control-Allow-Method", "POST")
    .raw_header("Access-Control-Allow-Headers", "*")
    .raw_header("Access-Control-Max-Age", "86400")
    .ok()
}

pub fn main(index: &Index) -> Result<(), Box<dyn std::error::Error>> {
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
    .manage(index)
    .mount("/", routes![add_records, debug_nodes, debug_nodes_cors])
    .attach(AdHoc::on_response(
      "CORS: allow all origins",
      |_, response| {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
      },
    ))
    .launch();

  Ok(())
}
