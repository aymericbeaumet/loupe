use memmap::{MmapMut, MmapOptions};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Arena<T> {
  mmap: MmapMut,
  offset: isize,
  t: PhantomData<T>,
  t_size: isize,
}

impl<T> Arena<T> {
  pub fn new(size: usize) -> Self {
    Self {
      mmap: MmapOptions::new().len(size).map_anon().unwrap(),
      offset: 0,
      t: PhantomData,
      t_size: std::mem::size_of::<T>() as isize,
    }
  }

  pub fn alloc(&mut self) -> *mut T {
    if (self.offset + self.t_size) as usize >= self.mmap.len() {
      panic!("OOM")
    }
    println!("[trace] alloc:offset={}", self.offset);
    let ptr = unsafe { self.mmap.as_mut_ptr().offset(self.offset) } as *mut _ as *mut T;
    self.offset += self.t_size;
    ptr
  }
}
