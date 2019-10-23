#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod child;
mod index;
mod parent;
mod store;

use index::Index;
use nix::unistd::{fork, ForkResult};
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut index = Index::new();
  match fork()? {
    ForkResult::Parent { .. } => Runtime::new()?.block_on(parent::main(&mut index)),
    ForkResult::Child => Runtime::new()?.block_on(child::main(&index)),
  }?;
  Ok(())
}
