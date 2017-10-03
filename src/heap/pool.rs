use alloc::allocator::Layout;
use collections::LinkedList;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

/// Heap memory pool.
pub struct Pool {
  free: LinkedList<()>,
  start: AtomicUsize,
  end: usize,
  size: usize,
}

/// Trait for pool queries.
pub trait PoolFit<T>
where
  T: Copy,
{
  /// Checks if the pool can fit the value.
  fn fits(&self, value: T) -> bool;
}

impl<'a> PoolFit<&'a Layout> for Pool {
  #[inline]
  fn fits(&self, layout: &Layout) -> bool {
    self.size >= layout.size()
  }
}

impl PoolFit<*mut u8> for Pool {
  #[inline]
  fn fits(&self, ptr: *mut u8) -> bool {
    self.end > ptr as usize
  }
}

impl Pool {
  /// Initializes new pool.
  pub const fn new(start: usize, size: usize, capacity: usize) -> Self {
    Self {
      free: LinkedList::new(),
      start: AtomicUsize::new(start),
      end: start + size * capacity,
      size,
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
    *self.start.get_mut() += offset;
    self.end += offset;
  }

  /// Allocates a block of memory.
  #[inline]
  pub fn alloc(&self) -> Option<*mut u8> {
    if let Some(ptr) = unsafe { self.free.pop_front_raw() } {
      Some(ptr as *mut _)
    } else {
      loop {
        let current = self.start.load(Relaxed);
        if current == self.end {
          return None;
        }
        let new = current + self.size;
        if self.start.compare_and_swap(current, new, Relaxed) == current {
          return Some(current as *mut _);
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
