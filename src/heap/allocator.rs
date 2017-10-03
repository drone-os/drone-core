use super::{Pool, PoolFit};
use alloc::allocator::{AllocErr, CannotReallocInPlace, Excess, Layout};
use core::{cmp, intrinsics, ptr};
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
  fn pool<T>(&self, value: T) -> Option<&Pool>
  where
    T: Copy,
    Pool: PoolFit<T>,
  {
    let (mut left, mut right) = (0, Self::POOL_COUNT);
    while right > left {
      let middle = left + ((right - left) >> 1);
      let pool = unsafe { self.get_pool_unchecked(middle) };
      if pool.fits(value) {
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
  fn alloc_with<F, T>(
    &self,
    layout: Layout,
    pool: Option<&Pool>,
    f: F,
  ) -> Result<T, AllocErr>
  where
    F: FnOnce(*mut u8, &Pool) -> T,
  {
    if let Some(pool) = pool {
      if let Some(ptr) = pool.alloc() {
        Ok(f(ptr, pool))
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
  fn realloc_with<F, T>(
    &self,
    ptr: *mut u8,
    layout: Layout,
    new_layout: Layout,
    f: F,
  ) -> Result<T, AllocErr>
  where
    F: FnOnce(*mut u8, &Pool) -> T,
  {
    let (new_size, old_size) = (new_layout.size(), layout.size());
    let new_pool = self.pool(&new_layout);
    let g = |ptr| {
      let new_pool = match new_pool {
        Some(pool) => pool,
        None => unsafe { intrinsics::unreachable() },
      };
      f(ptr, new_pool)
    };
    if layout.align() == new_layout.align() {
      if new_size < old_size {
        return Ok(g(ptr));
      } else if let Ok(()) =
        self.grow_in_place(ptr, layout.clone(), new_layout.clone())
      {
        return Ok(g(ptr));
      }
    }
    self
      .alloc_with(new_layout, new_pool, |ptr, _| ptr)
      .map(|new_ptr| {
        unsafe {
          ptr::copy_nonoverlapping(
            ptr as *const u8,
            new_ptr,
            cmp::min(old_size, new_size),
          );
        }
        self.dealloc(ptr, layout);
        g(new_ptr)
      })
  }

  #[doc(hidden)]
  #[inline]
  fn alloc(&self, layout: Layout) -> Result<*mut u8, AllocErr> {
    let pool = self.pool(&layout);
    self.alloc_with(layout, pool, |ptr, _| ptr)
  }

  #[doc(hidden)]
  #[inline]
  fn alloc_excess(&self, layout: Layout) -> Result<Excess, AllocErr> {
    let pool = self.pool(&layout);
    self.alloc_with(layout, pool, |ptr, pool| Excess(ptr, pool.size()))
  }

  #[doc(hidden)]
  #[inline]
  fn realloc(
    &self,
    ptr: *mut u8,
    layout: Layout,
    new_layout: Layout,
  ) -> Result<*mut u8, AllocErr> {
    self.realloc_with(ptr, layout, new_layout, |ptr, _| ptr)
  }

  #[doc(hidden)]
  #[inline]
  fn realloc_excess(
    &self,
    ptr: *mut u8,
    layout: Layout,
    new_layout: Layout,
  ) -> Result<Excess, AllocErr> {
    self.realloc_with(
      ptr,
      layout,
      new_layout,
      |ptr, pool| Excess(ptr, pool.size()),
    )
  }

  #[doc(hidden)]
  #[inline]
  fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
    match self.pool(ptr) {
      Some(pool) => pool.dealloc(ptr),
      None => unsafe { intrinsics::unreachable() },
    }
  }

  #[doc(hidden)]
  #[inline]
  fn usable_size(&self, layout: &Layout) -> (usize, usize) {
    match self.pool(layout) {
      Some(pool) => (0, pool.size()),
      None => unsafe { intrinsics::unreachable() },
    }
  }

  #[doc(hidden)]
  #[inline]
  fn grow_in_place(
    &self,
    _ptr: *mut u8,
    layout: Layout,
    new_layout: Layout,
  ) -> Result<(), CannotReallocInPlace> {
    match self.pool(&layout) {
      Some(pool) => if pool.fits(&new_layout) {
        Ok(())
      } else {
        Err(CannotReallocInPlace)
      },
      None => unsafe { intrinsics::unreachable() },
    }
  }

  #[doc(hidden)]
  #[inline]
  fn shrink_in_place(
    &self,
    _ptr: *mut u8,
    _layout: Layout,
    _new_layout: Layout,
  ) -> Result<(), CannotReallocInPlace> {
    Ok(())
  }

  #[doc(hidden)]
  #[inline]
  unsafe fn init(&mut self, start: &mut usize) {
    for i in 0..Self::POOL_COUNT {
      self.get_pool_unchecked_mut(i).init(start);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::mem;

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

  impl TestHeap {
    fn search_layout(&self, size: usize) -> Option<usize> {
      self
        .pool(&Layout::from_size_align(size, 4).unwrap())
        .map(Pool::size)
    }

    fn search_ptr(&self, ptr: usize) -> Option<usize> {
      self.pool(ptr as *mut u8).map(Pool::size)
    }
  }

  #[test]
  fn binary_search() {
    let heap = TestHeap {
      pools: [
        Pool::new(20, 2, 100),
        Pool::new(220, 5, 100),
        Pool::new(720, 8, 100),
        Pool::new(1520, 12, 100),
        Pool::new(2720, 16, 100),
        Pool::new(4320, 23, 100),
        Pool::new(6620, 38, 100),
        Pool::new(10420, 56, 100),
        Pool::new(16020, 72, 100),
        Pool::new(23220, 91, 100),
      ],
    };
    assert_eq!(heap.search_layout(1), Some(2));
    assert_eq!(heap.search_layout(2), Some(2));
    assert_eq!(heap.search_layout(15), Some(16));
    assert_eq!(heap.search_layout(16), Some(16));
    assert_eq!(heap.search_layout(17), Some(23));
    assert_eq!(heap.search_layout(91), Some(91));
    assert_eq!(heap.search_layout(92), None);
    assert_eq!(heap.search_ptr(0), Some(2));
    assert_eq!(heap.search_ptr(20), Some(2));
    assert_eq!(heap.search_ptr(219), Some(2));
    assert_eq!(heap.search_ptr(220), Some(5));
    assert_eq!(heap.search_ptr(719), Some(5));
    assert_eq!(heap.search_ptr(720), Some(8));
    assert_eq!(heap.search_ptr(721), Some(8));
    assert_eq!(heap.search_ptr(5000), Some(23));
    assert_eq!(heap.search_ptr(23220), Some(91));
    assert_eq!(heap.search_ptr(32319), Some(91));
    assert_eq!(heap.search_ptr(32320), None);
    assert_eq!(heap.search_ptr(50000), None);
  }

  #[test]
  fn allocations() {
    let mut heap = TestHeap {
      pools: [
        Pool::new(0, 2, 10),
        Pool::new(20, 5, 10),
        Pool::new(70, 8, 10),
        Pool::new(150, 12, 10),
        Pool::new(270, 16, 10),
        Pool::new(430, 23, 10),
        Pool::new(660, 38, 10),
        Pool::new(1040, 56, 10),
        Pool::new(1600, 72, 10),
        Pool::new(2320, 91, 10),
      ],
    };
    let mut m = [0u8; 3230];
    let o = &mut m as *mut _ as usize;
    let layout = Layout::from_size_align(32, 1).unwrap();
    unsafe {
      heap.init(mem::transmute(o));
      *heap.alloc(layout.clone()).unwrap() = 111;
      assert_eq!(m[660], 111);
      *heap.alloc(layout.clone()).unwrap() = 222;
      assert_eq!(m[698], 222);
      *heap.alloc(layout.clone()).unwrap() = 123;
      assert_eq!(m[736], 123);
      heap.dealloc((o + 660) as *mut _, layout.clone());
      assert_eq!(m[660], 0);
      heap.dealloc((o + 736) as *mut _, layout.clone());
      assert_eq!(*(&m[736] as *const _ as *const usize), o + 660);
      *heap.alloc(layout.clone()).unwrap() = 202;
      assert_eq!(m[736], 202);
      heap.dealloc((o + 698) as *mut _, layout.clone());
      assert_eq!(*(&m[698] as *const _ as *const usize), o + 660);
      heap.dealloc((o + 736) as *mut _, layout.clone());
      assert_eq!(*(&m[736] as *const _ as *const usize), o + 698);
    }
  }
}
