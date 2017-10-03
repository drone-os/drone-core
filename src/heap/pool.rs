use alloc::allocator::Layout;
use collections::LinkedList;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

/// Heap memory pool.
pub struct Pool {
  size: usize,
  start: usize,
  end: AtomicUsize,
  free: LinkedList<()>,
}

impl Pool {
  /// Initializes new pool.
  pub const fn new(start: usize, size: usize, capacity: usize) -> Self {
    Self {
      size,
      start,
      end: AtomicUsize::new(start + size * capacity),
      free: LinkedList::new(),
    }
  }

  /// Returns the pool size.
  pub fn size(&self) -> usize {
    self.size
  }

  /// Initializes the pool.
  ///
  /// # Safety
  ///
  /// Must be called exactly once and before using the pool.
  #[inline]
  pub unsafe fn init(&mut self, start: &mut usize) {
    let offset = start as *mut _ as usize;
    self.start += offset;
    *self.end.get_mut() += offset;
  }

  /// Checks if the pool can fit `layout`.
  #[inline]
  pub fn fits(&self, layout: &Layout) -> bool {
    self.size >= layout.size()
  }

  /// Allocates a block of memory.
  #[inline]
  pub fn alloc(&self) -> Option<*mut u8> {
    if let Some(ptr) = unsafe { self.free.pop_front_raw() } {
      Some(ptr as *mut _)
    } else {
      loop {
        let current = self.end.load(Relaxed);
        if current == self.start {
          return None;
        }
        let new = current - self.size;
        if self.end.compare_and_swap(current, new, Relaxed) == current {
          return Some(new as *mut _);
        }
      }
    }
  }

  /// Deallocates the block of memory referenced by `ptr`.
  #[inline]
  pub fn dealloc(&self, ptr: *mut u8) {
    unsafe {
      self.free.push_front_raw(ptr as *mut _);
    }
  }
}
