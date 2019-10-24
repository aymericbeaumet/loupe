use memmap::{MmapMut, MmapOptions};

#[derive(Debug)]
pub struct Store {
  mmap: MmapMut,
  offset: usize,
}

impl Store {
  pub fn new(len: usize) -> Self {
    Self {
      mmap: MmapOptions::new().len(len).map_anon().unwrap(),
      offset: 0,
    }
  }

  pub fn append(&mut self, slice: &[u8]) -> &[u8] {
    let appended = &mut self.mmap[self.offset..(self.offset + slice.len())];
    appended.copy_from_slice(&slice);
    self.offset += appended.len();
    appended
  }
}
