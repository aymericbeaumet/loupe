mod index;
mod store;

use index::Index;
use nix::unistd::{fork, ForkResult};
use rocksdb::DB;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut index = Index::new();
  match fork()? {
    ForkResult::Parent { .. } => Runtime::new()?.block_on(parent(&mut index)),
    ForkResult::Child => Runtime::new()?.block_on(child(&index)),
  };
  Ok(())
}

async fn parent(index: &mut Index) -> Result<(), Box<dyn std::error::Error>> {
  let db = DB::open_default("records.rocksdb")?;
  db.put(b"my key", b"my value")?;
  match db.get(b"my key") {
    Ok(Some(value)) => println!("retrieved value {}", value.to_utf8().unwrap()),
    Ok(None) => println!("value not found"),
    Err(e) => println!("operational problem encountered: {}", e),
  }
  db.delete(b"my key")?;
  Ok(())
}

async fn child(index: &Index) -> Result<(), Box<dyn std::error::Error>> {
  Ok(())
}
