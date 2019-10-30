use crate::index::Index;
use crate::record::Record;
use rocket::config::{Config, Environment};
use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Body {
  query: String,
}

#[post("/", format = "json", data = "<body>")]
fn query(index: State<Index>, body: Json<Body>) -> Json<Vec<Record>> {
  let records = index.query(&body.query).collect();
  Json(records)
}

pub fn main(index: Index) -> Result<(), Box<dyn std::error::Error>> {
  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(9292)
    .finalize()?;
  rocket::custom(config)
    .manage(index)
    .mount("/", routes![query])
    .launch();

  Ok(())
}
