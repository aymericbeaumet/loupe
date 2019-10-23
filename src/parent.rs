use crate::index::Index;
use rocket::config::{Config, Environment};
use rocksdb::DB;

#[get("/")]
fn index() -> &'static str {
  "Hello, parent!"
}

pub async fn main(index: &mut Index) -> Result<(), Box<dyn std::error::Error>> {
  let db = DB::open_default("records.rocksdb")?;
  db.put(b"my key", b"my value")?;
  match db.get(b"my key") {
    Ok(Some(value)) => println!("retrieved value {}", value.to_utf8().unwrap()),
    Ok(None) => println!("value not found"),
    Err(e) => println!("operational problem encountered: {}", e),
  }
  db.delete(b"my key")?;

  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(9191)
    .finalize()?;
  rocket::custom(config).mount("/", routes![index]).launch();

  Ok(())
}
