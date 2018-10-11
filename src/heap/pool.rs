use core::alloc::Layout;
use core::ptr::{self, NonNull};
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::*;

/// A lock-free fixed-size blocks allocator.
///
/// The `Pool` allows lock-free O(1) allocations, deallocations, and
/// initialization.
///
/// A `Pool` consists of `capacity` number of fixed-size blocks. It maintains a
/// *free list* of deallocated blocks.
pub struct Pool {
  /// Free List of previously allocated blocks.
  free: AtomicPtr<u8>,
  /// Growing inclusive pointer to the left edge of the uninitialized area.
  head: AtomicPtr<u8>,
  /// Non-inclusive right edge of the pool.
  edge: *mut u8,
  /// Size of blocks in the pool.
  size: usize,
}

impl Pool {
  /// Creates an empty `Pool`.
  ///
  /// The returned pool needs to be further initialized with [`init`] method.
  /// Resulting location of the pool should be the sum of `offset` argument
  /// provided to the current method and `start` argument for [`init`] method.
  ///
  /// [`init`]: Pool::init
  pub const fn new(offset: usize, size: usize, capacity: usize) -> Self {
    Self {
      free: AtomicPtr::new(ptr::null_mut()),
      head: AtomicPtr::new(offset as *mut u8),
      edge: (offset + size * capacity) as *mut u8,
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
  pub unsafe fn init(&mut self, start: &mut usize) {
    let offset = start as *mut _ as usize;
    let head = self.head.get_mut();
    *head = head.add(offset);
    self.edge = self.edge.add(offset);
  }

  /// Returns the pool size.
  #[inline(always)]
  pub fn size(&self) -> usize {
    self.size
  }

  /// Allocates a fixed-size block of memory. Returns `None` if the pool is
  /// exhausted.
  ///
  /// This operation should compute in O(1) time.
  pub fn alloc(&self) -> Option<NonNull<u8>> {
    unsafe { self.alloc_free().or_else(|| self.alloc_head()) }
  }

  /// Deallocates a fixed-size block of memory referenced by `ptr`.
  ///
  /// This operation should compute in O(1) time.
  ///
  /// # Safety
  ///
  /// * `ptr` should be a valid allocation.
  /// * `ptr` should not be used after deallocation.
  #[allow(clippy::cast_ptr_alignment)]
  pub unsafe fn dealloc(&self, ptr: NonNull<u8>) {
    loop {
      let head = self.free.load(Acquire);
      ptr::write(ptr.as_ptr() as *mut *mut u8, head);
      let next = ptr.as_ptr() as *mut u8;
      if self.free.compare_and_swap(head, next, AcqRel) == head {
        break;
      }
    }
  }

  #[allow(clippy::cast_ptr_alignment)]
  unsafe fn alloc_free(&self) -> Option<NonNull<u8>> {
    loop {
      let head = self.free.load(Acquire);
      if head.is_null() {
        break None;
      }
      let next = ptr::read(head as *const *mut u8);
      if self.free.compare_and_swap(head, next, AcqRel) == head {
        break Some(NonNull::new_unchecked(head));
      }
    }
  }

  unsafe fn alloc_head(&self) -> Option<NonNull<u8>> {
    loop {
      let head = self.head.load(Relaxed);
      if head == self.edge {
        break None;
      }
      let next = head.add(self.size);
      if self.head.compare_and_swap(head, next, Relaxed) == head {
        break Some(NonNull::new_unchecked(head));
      }
    }
  }
}

pub trait Fits: Copy {
  fn fits(self, pool: &Pool) -> bool;
}

impl<'a> Fits for &'a Layout {
  #[inline(always)]
  fn fits(self, pool: &Pool) -> bool {
    self.size() <= pool.size
  }
}

impl Fits for NonNull<u8> {
  #[inline(always)]
  fn fits(self, pool: &Pool) -> bool {
    (self.as_ptr() as *mut u8) < pool.edge
  }
}
