use crate::index::Index;
use crate::record::Record;
use serde::Deserialize;
use std::collections::HashMap;

pub type RecordsParams = Vec<Record>;

#[derive(Deserialize)]
pub struct QueryParams {
  query: String,
}

pub fn add_records(params: RecordsParams, index: Index) -> impl warp::reply::Reply {
  for record in params.iter() {
    let record_as_string = record.to_string();
    let record_as_bytes = record_as_string.as_bytes();
    index.add_record_slice(record_as_bytes);
  }
  warp::reply::json(&params)
}

pub fn debug_nodes(params: QueryParams, index: Index) -> impl warp::reply::Reply {
  let nodes: HashMap<_, _> = index
    .query_nodes(&params.query)
    .map(|(string, node)| (string, *node)) // TODO: remove the node copy
    .collect();
  warp::reply::json(&nodes)
}

pub fn query_records(params: QueryParams, index: Index) -> impl warp::reply::Reply {
  let records: Vec<Record> = index.query_records(&params.query).collect();
  warp::reply::json(&records)
}
