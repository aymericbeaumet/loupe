use memmap::{MmapMut, MmapOptions};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Store<T> {
  size_of: usize,
  mmap: MmapMut,
  _phantom: PhantomData<T>,
}

impl<T> Store<T> {
  pub fn new(len: usize) -> Self {
    let size_of = std::mem::size_of::<T>();
    Self {
      size_of,
      mmap: MmapOptions::new().len(len * size_of).map_anon().unwrap(),
      _phantom: PhantomData,
    }
  }

  pub fn chunk(&self, index: usize) -> &T {
    let ptr = self.mmap.as_ptr();
    let ptr = unsafe { ptr.offset(self.offset(index)) } as *const _ as *const T;
    unsafe { &*ptr }
  }

  pub fn chunk_mut(&mut self, index: usize) -> &mut T {
    let ptr = self.mmap.as_mut_ptr();
    let ptr = unsafe { ptr.offset(self.offset(index)) } as *mut _ as *mut T;
    unsafe { &mut *ptr }
  }

  fn offset(&self, index: usize) -> isize {
    (index * self.size_of) as isize
  }
}
