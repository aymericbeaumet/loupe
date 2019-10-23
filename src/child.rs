use crate::index::Index;
use rocket::config::{Config, Environment};
use rocket::State;
use rocket_contrib::json::Json;
use rocksdb::DB;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Body {
  query: String,
}

#[post("/", format = "json", data = "<body>")]
fn index(db: State<DB>, body: Json<Body>) -> String {
  match db.get(body.into_inner().query.as_bytes()) {
    Ok(Some(value)) => value.to_utf8().unwrap().to_owned(),
    Ok(None) => format!("value not found"),
    Err(e) => format!("operational problem encountered: {}", e),
  }
}

pub fn main(index: &Index) -> Result<(), Box<dyn std::error::Error>> {
  // TODO: remove and read from the index instead
  let db = DB::open_default("records.rocksdb")?;

  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(9292)
    .finalize()?;
  rocket::custom(config)
    .manage(db)
    .mount("/", routes![index])
    .launch();

  Ok(())
}
