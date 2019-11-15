use crate::index::{Index, Node};
use crate::record::Record;
use rocksdb::{WriteBatch, DB};
use serde::Deserialize;
use std::collections::HashMap;
use warp::{Filter, Rejection};

/// public returns a service exposing the public routes.
pub fn public(index: Index) -> impl Filter<Extract = (&'static str,), Error = Rejection> + Clone {
  let index = warp::any().map(|| index);
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

  query_records_post.or(query_records_get).with(cors)
}

/// restricted returns a service exposing the restricted routes.
pub fn restricted(
  db: rocksdb::DB,
  index: Index,
) -> impl Filter<Extract = (&'static str,), Error = Rejection> + Clone {
  let db = warp::any().map(|| db);
  let index = warp::any().map(|| index);
  let cors = warp::cors()
    .allow_origin("http://localhost:1234")
    .allow_methods(vec!["POST", "GET"]);

  let add_records_post = warp::post()
    .and(warp::path("/records"))
    .and(warp::body::json())
    .and(db)
    .and(index)
    .map(add_records);

  let debug_nodes_get = warp::get()
    .and(warp::path("/debug/nodes"))
    .and(warp::query())
    .and(index)
    .map(debug_nodes);

  add_records_post.or(debug_nodes_get).with(cors)
}

/*
 * routes handlers implementations
*/

// TODO: add record to the index as batch after they have been written to disk
fn add_records(records: Vec<Record>, db: DB, index: Index) -> impl warp::reply::Reply {
  let mut batch = WriteBatch::default();
  for record in records {
    let id_as_bytes = record.id.to_ne_bytes();
    let record_as_string = record.to_string();
    let record_as_bytes = record_as_string.as_bytes();
    batch.put(id_as_bytes, record_as_bytes).unwrap();
    index.add_record_slice(record_as_bytes);
  }
  db.write(batch).unwrap();
  warp::reply::json(&records)
}

#[derive(Debug, Deserialize)]
struct DebugNodesParams {
  query: String,
}

fn debug_nodes(params: DebugNodesParams, index: Index) -> impl warp::reply::Reply {
  let nodes: HashMap<String, &Node> = index.query_nodes(&params.query).collect();
  warp::reply::json(&nodes)
}

#[derive(Debug, Deserialize)]
struct QueryRecordsParams {
  query: String,
}

fn query_records(params: QueryRecordsParams, index: Index) -> impl warp::reply::Reply {
  let records: Vec<Record> = index.query_records(&params.query).collect();
  warp::reply::json(&records)
}
