#[macro_use]
extern crate lazy_static;

mod arena;
mod index;
mod record;
mod services;
mod tokenizer;

use rocksdb::DB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let index = index::Index::new();
  let db = DB::open_default("records.rocksdb")?;

  for (_, record) in db.iterator(rocksdb::IteratorMode::Start) {
    index.add_record_slice(&record);
  }

  futures::join!(
    async {
      warp::serve(services::public(index)).run(([127, 0, 0, 1], 9191));
    },
    async {
      warp::serve(services::restricted(db, index)).run(([127, 0, 0, 1], 9292));
    }
  );

  Ok(())
}
