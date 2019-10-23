use crate::index::Index;
use rocket::config::{Config, Environment};

#[get("/")]
fn index() -> &'static str {
  "Hello, child!"
}

pub async fn main(index: &Index) -> Result<(), Box<dyn std::error::Error>> {
  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(9292)
    .finalize()?;
  rocket::custom(config).mount("/", routes![index]).launch();

  Ok(())
}
