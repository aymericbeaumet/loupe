use crate::handlers::*;
use crate::index::Index;
use warp::Filter;

pub fn private(index: Index) -> impl futures::future::Future {
  let index = warp::any().map(move || index);

  let add_records_post = warp::post()
    .and(path!("records"))
    .and(warp::body::json())
    .and(index)
    .map(add_records);
  let debug_nodes_get = warp::get()
    .and(path!("debug" / "nodes"))
    .and(warp::query())
    .and(index)
    .map(debug_nodes);

  let routes = add_records_post.or(debug_nodes_get);

  let cors = warp::cors()
    .allow_origin("http://localhost:1234")
    .allow_methods(vec!["POST", "GET"]);
  let log = warp::log("services::private");

  warp::serve(routes.with(cors).with(log)).run(([127, 0, 0, 1], 9191))
}

pub fn public(index: Index) -> impl futures::future::Future {
  let index = warp::any().map(move || index);

  let query_records_post = warp::post()
    .and(warp::body::json())
    .and(index)
    .map(query_records);
  let query_records_get = warp::get().and(warp::query()).and(index).map(query_records);

  let routes = query_records_post.or(query_records_get);

  let cors = warp::cors().allow_any_origin().allow_methods(vec!["GET"]);
  let log = warp::log("services::public");

  warp::serve(routes.with(cors).with(log)).run(([127, 0, 0, 1], 9292))
}
