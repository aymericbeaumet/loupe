use crate::index::Index;
use rocket::config::{Config, Environment};
use rocket_contrib::json::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Body {
  query: String,
}

#[post("/", format = "json", data = "<body>")]
fn query(body: Json<Body>) -> &'static str {
  "query"
}

pub fn main(index: &Index) -> Result<(), Box<dyn std::error::Error>> {
  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(9292)
    .finalize()?;
  rocket::custom(config).mount("/", routes![query]).launch();

  Ok(())
}
