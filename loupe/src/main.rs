#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;

mod arena;
mod child;
mod index;
mod parent;
mod record;
mod tokenizer;

fn main() {
  let index = index::Index::new();
  crossbeam::thread::scope(|scope| {
    scope.spawn(|_| {
      parent::main(&index);
    });
    scope.spawn(|_| {
      child::main(&index);
    });
  })
  .unwrap();
}
