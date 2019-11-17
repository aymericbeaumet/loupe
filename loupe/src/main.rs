#![recursion_limit = "512"]

#[macro_use]
extern crate lazy_static;

mod arena;
mod handlers;
mod index;
mod record;
mod tokenizer;

use handlers::*;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let index = index::Index::new();
  let index = warp::any().map(move || index);

  futures::join!(
    {
      let cors = warp::cors().allow_any_origin().allow_methods(vec!["GET"]);

      let query_records_post = warp::post()
        .and(warp::path("/"))
        .and(warp::body::json())
        .and(index)
        .map(query_records);

      let query_records_get = warp::get()
        .and(warp::path("/"))
        .and(warp::query())
        .and(index)
        .map(query_records);

      let service = query_records_post.or(query_records_get).with(cors);

      warp::serve(service).run(([127, 0, 0, 1], 9191))
    },
    {
      let cors = warp::cors()
        .allow_origin("http://localhost:1234")
        .allow_methods(vec!["POST", "GET"]);

      let add_records_post = warp::post()
        .and(warp::path("/records"))
        .and(warp::body::json())
        .and(index)
        .map(add_records);

      let debug_nodes_get = warp::get()
        .and(warp::path("/debug/nodes"))
        .and(warp::query())
        .and(index)
        .map(debug_nodes);

      let service = add_records_post.or(debug_nodes_get).with(cors);

      warp::serve(service).run(([127, 0, 0, 1], 9292))
    }
  );

  Ok(())
}
