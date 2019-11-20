use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::marker::PhantomData;
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicU32, Ordering};

// TODO: we will need this someday, right now we just make sure the address 0x0
// is not publicly available outside the arena by allocating an empty header at
// the beginning.
struct AllocHeader(u128);

impl AllocHeader {
  fn new() -> Self {
    Self(0x42)
  }
}

pub struct Arena {
  layout: Layout,
  ptr: *mut u8,
  offset: AtomicU32,
}

unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

// Aligned on 64 bits as quadword RW are guaranteed to be carried atomically
// since Pentium processors (see: 8.1.1 Guaranteed Atomic Operations,
// https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf)
impl Arena {
  pub fn new() -> Self {
    let layout = Layout::from_size_align(100_000_000, 64).expect("Unable to align layout");
    let arena = Self {
      layout,
      ptr: unsafe { alloc_zeroed(layout) },
      offset: AtomicU32::new(0),
    };
    arena.alloc_copy(AllocHeader::new());
    arena
  }

  // Value

  #[inline(always)]
  pub fn alloc<T>(&self) -> ArenaTypeKey<T> {
    let size = std::mem::size_of::<T>() as u32;
    let offset = self.offset.fetch_add(size, Ordering::SeqCst);
    ArenaTypeKey::new(offset)
  }

  #[inline(always)]
  pub fn alloc_copy<T>(&self, value: T) -> ArenaTypeKey<T> {
    let key = self.alloc::<T>();
    unsafe {
      let dst = self.ptr.offset(key.offset as isize) as *mut T;
      ptr::write(dst, value);
    };
    key
  }

  #[inline(always)]
  pub fn get<T>(&self, key: ArenaTypeKey<T>) -> Option<&T> {
    if !key.is_null() {
      Some(unsafe { self.get_unchecked(key) })
    } else {
      None
    }
  }

  #[inline(always)]
  pub unsafe fn get_unchecked<T>(&self, key: ArenaTypeKey<T>) -> &T {
    let src = self.ptr.offset(key.offset as isize);
    &*(src as *const T)
  }

  pub fn free<T>(&self, key: ArenaTypeKey<T>, when: u32) {
    // TODO
  }

  // Slice

  #[inline(always)]
  pub fn alloc_slice<T>(&self, src: &[T]) -> ArenaSliceKey<T> {
    let size = std::mem::size_of_val(src) as u32;
    let offset = self.offset.fetch_add(size, Ordering::SeqCst);
    ArenaSliceKey::new(offset, src.len() as u32)
  }

  #[inline(always)]
  pub fn alloc_slice_copy<T>(&self, src: &[T]) -> ArenaSliceKey<T> {
    let key = self.alloc_slice(src);
    unsafe {
      let dst = self.ptr.offset(key.offset as isize) as *mut _;
      ptr::copy_nonoverlapping(src.as_ptr(), dst, key.len as usize);
      slice::from_raw_parts_mut(dst, src.len())
    };
    key
  }

  #[inline(always)]
  pub fn get_slice<T>(&self, key: &ArenaSliceKey<T>) -> Option<&[T]> {
    if !key.is_null() {
      Some(unsafe { self.get_slice_unchecked(key) })
    } else {
      None
    }
  }

  #[inline(always)]
  pub unsafe fn get_slice_unchecked<T>(&self, key: &ArenaSliceKey<T>) -> &[T] {
    let src = self.ptr.offset(key.offset as isize) as *const _;
    std::slice::from_raw_parts(src, key.len as usize)
  }
}

impl Drop for Arena {
  fn drop(&mut self) {
    unsafe { dealloc(self.ptr, self.layout) };
  }
}

#[derive(Debug, Copy, Clone)]
pub struct ArenaTypeKey<T> {
  offset: u32,
  _t: PhantomData<T>,
}

impl<T> ArenaTypeKey<T> {
  pub fn new(offset: u32) -> Self {
    Self {
      offset,
      _t: PhantomData,
    }
  }

  #[inline(always)]
  pub fn is_null(&self) -> bool {
    self.offset == 0
  }
}

#[derive(Debug, Copy, Clone)]
pub struct ArenaSliceKey<T> {
  offset: u32,
  len: u32,
  _t: PhantomData<T>,
}

impl<T> ArenaSliceKey<T> {
  pub fn new(offset: u32, len: u32) -> Self {
    Self {
      offset,
      len,
      _t: PhantomData,
    }
  }

  #[inline(always)]
  pub fn is_null(&self) -> bool {
    self.offset == 0 || self.len == 0
  }
}

impl<T> PartialEq for ArenaSliceKey<T> {
  fn eq(&self, other: &Self) -> bool {
    self.offset == other.offset && self.len == other.len
  }
}
