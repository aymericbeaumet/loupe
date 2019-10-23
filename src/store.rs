use memmap::MmapOptions;

#[derive(Debug)]
pub struct Store {
  mmap_ptr: *const u8,
  mmap_ptr_mut: *mut u8,
}

impl Store {
  pub fn new(len: usize) -> Self {
    let mut mmap = MmapOptions::new().len(len).map_anon().unwrap();
    Self {
      mmap_ptr: mmap.as_ptr(),
      mmap_ptr_mut: mmap.as_mut_ptr(),
    }
  }

  pub fn chunk<T>(&self, offset: isize) -> &T {
    let size = std::mem::size_of::<T>();
    let ptr = unsafe { self.mmap_ptr.offset(offset) } as *const _ as *const T;
    unsafe { &*ptr }
  }

  pub fn chunk_mut<T>(&mut self, offset: isize) -> &mut T {
    let size = std::mem::size_of::<T>();
    let ptr = unsafe { self.mmap_ptr_mut.offset(offset) } as *mut _ as *mut T;
    unsafe { &mut *ptr }
  }
}
