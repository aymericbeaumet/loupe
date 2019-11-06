#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod arena;
mod child;
mod index;
mod normalizer;
mod parent;
mod record;

use index::Index;
use nix::unistd::{fork, ForkResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let index = Index::new();
  match fork()? {
    ForkResult::Parent { .. } => parent::main(index),
    ForkResult::Child => child::main(index),
  }?;
  Ok(())
}
