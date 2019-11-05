use crate::index::Index;
use crate::record::Record;
use rocket::config::{Config, Environment};
use rocket::fairing::AdHoc;
use rocket::http::{Header, Status};
use rocket::response::Response;
use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Body {
  query: String,
}

#[post("/", format = "json", data = "<body>")]
fn query(index: State<Index>, body: Json<Body>) -> Json<Vec<Record>> {
  let records = index.query_records(&body.query.trim()).collect();
  Json(records)
}

#[options("/")]
fn query_cors() -> Result<Response<'static>, Status> {
  Response::build()
    .status(Status::NoContent)
    .raw_header("Access-Control-Allow-Method", "POST")
    .raw_header("Access-Control-Allow-Headers", "*")
    .raw_header("Access-Control-Max-Age", "86400")
    .ok()
}

pub fn main(index: Index) -> Result<(), Box<dyn std::error::Error>> {
  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(9292)
    .finalize()?;

  rocket::custom(config)
    .manage(index)
    .mount("/", routes![query, query_cors])
    .attach(AdHoc::on_response(
      "CORS: allow all origins",
      |_, response| {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
      },
    ))
    .launch();

  Ok(())
}
