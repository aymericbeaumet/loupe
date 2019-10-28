use memmap::{MmapMut, MmapOptions};
use std::sync::atomic::{AtomicIsize, Ordering};

#[derive(Debug)]
pub struct Arena {
  mmap: MmapMut,
  offset: AtomicIsize,
}

impl Arena {
  pub fn new(size: usize) -> Self {
    Self {
      mmap: MmapOptions::new().len(size).map_anon().unwrap(),
      offset: AtomicIsize::new(0),
    }
  }

  pub fn alloc(&self, size: isize) -> Result<*mut u8, ArenaError> {
    let offset = self.offset.fetch_add(size, Ordering::SeqCst);
    if (offset + size) as usize >= self.mmap.len() {
      Err("Out of memory".into())
    } else {
      Ok(unsafe { self.mmap.as_ptr().offset(offset) as *mut u8 })
    }
  }

  pub fn alloc_type<T>(&self) -> Result<*mut T, ArenaError> {
    Ok(self.alloc(std::mem::size_of::<T>() as isize)? as *mut _ as *mut T)
  }
}

#[derive(Debug)]
pub struct ArenaError(String);

impl From<&str> for ArenaError {
  fn from(s: &str) -> Self {
    Self(s.to_owned())
  }
}

impl std::fmt::Display for ArenaError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for ArenaError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    None
  }
}
