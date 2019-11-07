#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate rocket;

mod arena;
mod child;
mod index;
mod parent;
mod record;
mod tokenizer;

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
