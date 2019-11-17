use crate::index::Index;
use crate::record::Record;
use serde::Deserialize;
use std::collections::HashMap;

pub type AddRecordsParams = Vec<Record>;

pub fn add_records(params: AddRecordsParams, index: Index) -> impl warp::reply::Reply {
  for record in params.iter() {
    let record_as_string = record.to_string();
    let record_as_bytes = record_as_string.as_bytes();
    index.add_record_slice(record_as_bytes);
  }
  warp::reply::json(&params)
}

#[derive(Debug, Deserialize)]
pub struct DebugNodesParams {
  query: String,
}

pub fn debug_nodes(params: DebugNodesParams, index: Index) -> impl warp::reply::Reply {
  let nodes: HashMap<_, _> = index
    .query_nodes(&params.query)
    .map(|(string, node)| (string, *node)) // TODO: remove this node copy
    .collect();
  warp::reply::json(&nodes)
}

#[derive(Debug, Deserialize)]
pub struct QueryRecordsParams {
  query: String,
}

pub fn query_records(params: QueryRecordsParams, index: Index) -> impl warp::reply::Reply {
  let records: Vec<Record> = index.query_records(&params.query).collect();
  warp::reply::json(&records)
}
