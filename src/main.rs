#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod child;
mod index;
mod parent;
mod store;

use index::Index;
use nix::unistd::{fork, ForkResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut index = Index::new();
  match fork()? {
    ForkResult::Parent { .. } => parent::main(&mut index),
    ForkResult::Child => child::main(&index),
  }?;
  Ok(())
}
