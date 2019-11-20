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
async fn main() {
  pretty_env_logger::init();

  let index = index::Index::new();
  let private_service = services::private(index);
  let public_service = services::public(index);

  futures::join!(private_service, public_service);
}
