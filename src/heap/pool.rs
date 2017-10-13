//! A lock-free fixed-size blocks allocator.
//!
//! See [`Pool`] for more details.
//!
//! [`Pool`]: struct.Pool.html

use alloc::allocator::Layout;
use collections::LinkedList;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::*;
use mem::ManuallyDrop;

/// A lock-free fixed-size blocks allocator.
///
/// The `Pool` allows lock-free O(1) allocations, deallocations, and
/// initialization.
///
/// A `Pool` consists of `capacity` number of fixed-size blocks. It maintains a
/// *free list* of deallocated blocks.
pub struct Pool<T> {
  /// Free List of previously allocated blocks.
  free: ManuallyDrop<LinkedList<()>>,
  /// Growing inclusive pointer to the left edge of the uninitialized area.
  head: AtomicPtr<T>,
  /// Non-inclusive right edge of the pool.
  edge: *mut T,
  /// Size of blocks in the pool.
  size: usize,
}

/// Trait for values that can be checked against a `Pool`.
pub trait Fits<T>
where
  Self: Copy,
{
  /// The method tests that `self` fits `pool`.
  fn fits(self, pool: &Pool<T>) -> bool;
}

impl<'a, T> Fits<T> for &'a Layout {
  #[inline]
  fn fits(self, pool: &Pool<T>) -> bool {
    self.size() <= pool.size
  }
}

impl<T> Fits<T> for *mut T {
  #[inline]
  fn fits(self, pool: &Pool<T>) -> bool {
    self < pool.edge
  }
}

impl<T> Pool<T> {
  /// Creates an empty `Pool`.
  ///
  /// The returned pool needs to be further initialized with [`init`] method.
  /// Resulting location of the pool should be the sum of `offset` argument
  /// provided to the current method and `start` argument for [`init`] method.
  ///
  /// [`init`]: struct.Pool.html#method.init
  pub const fn new(offset: usize, size: usize, capacity: usize) -> Self {
    Self {
      free: ManuallyDrop::new(LinkedList::new()),
      head: AtomicPtr::new(offset as *mut T),
      edge: (offset + size * capacity) as *mut T,
      size,
    }
  }

  /// Initializes the pool with `start` address.
  ///
  /// This operation should compute in O(1) time.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Must be called before using the pool.
  #[inline]
  pub unsafe fn init(&mut self, start: &mut usize) {
    let offset = start as *mut _ as usize;
    let head = self.head.get_mut();
    *head = head.add(offset);
    self.edge = self.edge.add(offset);
  }

  /// Returns the pool size.
  pub fn size(&self) -> usize {
    self.size
  }

  /// Allocates a fixed-size block of memory. Returns `None` if the pool is
  /// exhausted.
  ///
  /// This operation should compute in O(1) time.
  #[inline]
  pub fn alloc(&self) -> Option<*mut T> {
    let ptr = unsafe { self.free.pop_raw() };
    if !ptr.is_null() {
      Some(ptr)
    } else {
      loop {
        let current = self.head.load(Relaxed);
        if current == self.edge {
          return None;
        }
        let new = unsafe { current.add(self.size) };
        if self.head.compare_and_swap(current, new, Relaxed) == current {
          return Some(current);
        }
      }
    }
  }

  /// Deallocates a fixed-size block of memory referenced by `ptr`.
  ///
  /// This operation should compute in O(1) time.
  ///
  /// # Safety
  ///
  /// `ptr` should not be used after deallocation.
  #[inline]
  pub unsafe fn dealloc(&self, ptr: *mut T) {
    self.free.push_raw(ptr);
  }
}
