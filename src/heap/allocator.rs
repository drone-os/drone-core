use super::Pool;
use alloc::allocator::{AllocErr, Layout};
use core::slice::SliceIndex;

/// Heap allocator.
///
/// It should store pools sorted by size.
pub trait Allocator {
  /// Number of memory pools.
  const POOL_COUNT: usize;

  /// Returns a reference to a pool or subslice, without doing bounds checking.
  unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
  where
    I: SliceIndex<[Pool]>;

  /// Returns a mutable reference to a pool or subslice, without doing bounds
  /// checking.
  unsafe fn get_pool_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
  where
    I: SliceIndex<[Pool]>;

  #[doc(hidden)]
  #[inline]
  unsafe fn init(&mut self, start: &mut usize) {
    for i in 0..Self::POOL_COUNT {
      self.get_pool_unchecked_mut(i).init(start);
    }
  }

  /// Returns a matching pool for `layout`.
  #[inline]
  fn pool(&self, layout: &Layout) -> Option<&Pool> {
    let (mut left, mut right) = (0, Self::POOL_COUNT);
    while right > left {
      let middle = left + ((right - left) >> 1);
      let pool = unsafe { self.get_pool_unchecked(middle) };
      if pool.fits(layout) {
        right = middle;
      } else {
        left = middle + 1;
      }
    }
    if left < Self::POOL_COUNT {
      Some(unsafe { self.get_pool_unchecked(left) })
    } else {
      None
    }
  }

  #[doc(hidden)]
  #[inline]
  fn alloc(&self, layout: Layout) -> Result<*mut u8, AllocErr> {
    if let Some(pool) = self.pool(&layout) {
      if let Some(ptr) = pool.alloc() {
        Ok(ptr)
      } else {
        Err(AllocErr::Exhausted { request: layout })
      }
    } else {
      Err(AllocErr::Unsupported {
        details: "No memory pool for the given size",
      })
    }
  }

  #[doc(hidden)]
  #[inline]
  fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    if let Some(pool) = self.pool(&layout) {
      pool.dealloc(ptr);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct TestHeap {
    pools: [Pool; 10],
  }

  impl Allocator for TestHeap {
    const POOL_COUNT: usize = 10;

    unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
    where
      I: SliceIndex<[Pool]>,
    {
      self.pools.get_unchecked(index)
    }

    unsafe fn get_pool_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
    where
      I: SliceIndex<[Pool]>,
    {
      self.pools.get_unchecked_mut(index)
    }
  }

  static TEST_HEAP: TestHeap = TestHeap {
    pools: [
      Pool::new(0, 2, 100),
      Pool::new(0, 5, 100),
      Pool::new(0, 8, 100),
      Pool::new(0, 12, 100),
      Pool::new(0, 16, 100),
      Pool::new(0, 23, 100),
      Pool::new(0, 38, 100),
      Pool::new(0, 56, 100),
      Pool::new(0, 72, 100),
      Pool::new(0, 91, 100),
    ],
  };

  fn search(size: usize) -> Option<usize> {
    TEST_HEAP
      .pool(&Layout::from_size_align(size, 4).unwrap())
      .map(Pool::size)
  }

  #[test]
  fn binary_search() {
    assert_eq!(search(1), Some(2));
    assert_eq!(search(2), Some(2));
    assert_eq!(search(15), Some(16));
    assert_eq!(search(16), Some(16));
    assert_eq!(search(17), Some(23));
    assert_eq!(search(91), Some(91));
    assert_eq!(search(92), None);
  }
}
