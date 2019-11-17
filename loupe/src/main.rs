#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate warp;

mod arena;
mod handlers;
mod index;
mod record;
mod services;
mod tokenizer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  pretty_env_logger::init();

  let index = index::Index::new();
  let public_service = services::public(index);
  let private_service = services::private(index);

  futures::join!(public_service, private_service); // TODO: main should return this future's result

  Ok(())
}
