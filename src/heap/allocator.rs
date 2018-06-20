use super::pool::{Fits, Pool};
use core::alloc::{AllocErr, CannotReallocInPlace, Excess, Layout};
use core::ptr::NonNull;
use core::slice::SliceIndex;
use core::{cmp, ptr};

/// A lock-free allocator that composes multiple memory pools.
///
/// An `Allocator` maintains a sort-order of its pools, so they can be
/// effectively accessed with [`binary_search`](Allocator::binary_search).
pub trait Allocator {
  /// Number of memory pools.
  const POOL_COUNT: usize;

  /// Initializes the pools with `start` address.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Must be called before using the allocator.
  /// * `start` must be word-aligned.
  #[inline(always)]
  unsafe fn init(&mut self, start: &mut usize) {
    for i in 0..Self::POOL_COUNT {
      self.get_pool_unchecked_mut(i).init(start);
    }
  }

  /// Returns a reference to a pool or subslice, without doing bounds checking.
  unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
  where
    I: SliceIndex<[Pool]>;

  /// Returns a mutable reference to a pool or subslice, without doing bounds
  /// checking.
  unsafe fn get_pool_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
  where
    I: SliceIndex<[Pool]>;

  /// Binary searches the pools for a least-sized one which fits `value`.
  #[inline(always)]
  fn binary_search<T: Fits>(&self, value: T) -> usize {
    let (mut left, mut right) = (0, Self::POOL_COUNT);
    while right > left {
      let middle = left + ((right - left) >> 1);
      let pool = unsafe { self.get_pool_unchecked(middle) };
      if value.fits(pool) {
        right = middle;
      } else {
        left = middle + 1;
      }
    }
    left
  }

  #[doc(hidden)]
  #[inline(always)]
  unsafe fn alloc_with<F, T>(&self, layout: Layout, f: F) -> Result<T, AllocErr>
  where
    F: FnOnce(NonNull<u8>, &Pool) -> T,
  {
    let mut pool_idx = self.binary_search(&layout);
    if pool_idx == Self::POOL_COUNT {
      return Err(AllocErr);
    }
    loop {
      let pool = self.get_pool_unchecked(pool_idx);
      if let Some(ptr) = pool.alloc() {
        return Ok(f(ptr, pool));
      }
      pool_idx += 1;
      if pool_idx == Self::POOL_COUNT {
        return Err(AllocErr);
      }
    }
  }

  #[doc(hidden)]
  #[cfg_attr(feature = "cargo-clippy", allow(cast_ptr_alignment))]
  #[inline(always)]
  unsafe fn realloc_with<F, T>(
    &self,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
    f: F,
  ) -> Result<T, AllocErr>
  where
    F: Fn(NonNull<u8>, &Pool) -> T,
  {
    let old_size = layout.size();
    let in_place = if new_size < old_size {
      true
    } else if let Ok(()) = self.grow_in_place(ptr, layout, new_size) {
      true
    } else {
      false
    };
    let new_layout =
      Layout::from_size_align_unchecked(new_size, layout.align());
    if in_place {
      Ok(f(
        ptr,
        self.get_pool_unchecked(self.binary_search(&new_layout)),
      ))
    } else {
      self.alloc_with(new_layout, |new_ptr, pool| {
        ptr::copy_nonoverlapping(
          ptr.as_ptr() as *const usize,
          new_ptr.as_ptr() as *mut usize,
          cmp::min(old_size, new_size),
        );
        self.dealloc(ptr, layout);
        f(new_ptr, pool)
      })
    }
  }

  #[doc(hidden)]
  #[inline(always)]
  unsafe fn alloc(&self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
    self.alloc_with(layout, |ptr, _| ptr)
  }

  #[doc(hidden)]
  #[inline(always)]
  unsafe fn alloc_excess(&self, layout: Layout) -> Result<Excess, AllocErr> {
    self.alloc_with(layout, |ptr, pool| Excess(ptr, pool.size()))
  }

  #[doc(hidden)]
  #[inline(always)]
  unsafe fn realloc(
    &self,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
  ) -> Result<NonNull<u8>, AllocErr> {
    self.realloc_with(ptr, layout, new_size, |ptr, _| ptr)
  }

  #[doc(hidden)]
  #[inline(always)]
  unsafe fn realloc_excess(
    &self,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
  ) -> Result<Excess, AllocErr> {
    self
      .realloc_with(ptr, layout, new_size, |ptr, pool| Excess(ptr, pool.size()))
  }

  #[doc(hidden)]
  #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
  #[inline(always)]
  unsafe fn dealloc(&self, ptr: NonNull<u8>, _layout: Layout) {
    let pool_idx = self.binary_search(ptr);
    let pool = self.get_pool_unchecked(pool_idx);
    pool.dealloc(ptr);
  }

  #[doc(hidden)]
  #[inline(always)]
  unsafe fn usable_size(&self, layout: &Layout) -> (usize, usize) {
    let pool_idx = self.binary_search(layout);
    let pool = self.get_pool_unchecked(pool_idx);
    (0, pool.size())
  }

  #[doc(hidden)]
  #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
  #[inline(always)]
  unsafe fn grow_in_place(
    &self,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
  ) -> Result<(), CannotReallocInPlace> {
    let pool_idx = self.binary_search(ptr);
    let pool = self.get_pool_unchecked(pool_idx);
    let new_layout =
      Layout::from_size_align_unchecked(new_size, layout.align());
    if new_layout.fits(pool) {
      Ok(())
    } else {
      Err(CannotReallocInPlace)
    }
  }

  #[doc(hidden)]
  #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
  #[inline(always)]
  unsafe fn shrink_in_place(
    &self,
    _ptr: NonNull<u8>,
    _layout: Layout,
    _new_size: usize,
  ) -> Result<(), CannotReallocInPlace> {
    Ok(())
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
      let pool_idx =
        self.binary_search(&Layout::from_size_align(size, 4).unwrap());
      if pool_idx < TestHeap::POOL_COUNT {
        unsafe { Some(self.get_pool_unchecked(pool_idx).size()) }
      } else {
        None
      }
    }

    fn search_ptr(&self, ptr: usize) -> Option<usize> {
      let pool_idx =
        self.binary_search(unsafe { NonNull::new_unchecked(ptr as *mut u8) });
      if pool_idx < TestHeap::POOL_COUNT {
        unsafe { Some(self.get_pool_unchecked(pool_idx).size()) }
      } else {
        None
      }
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
      *(heap.alloc(layout.clone()).unwrap().as_ptr() as *mut u8) = 111;
      assert_eq!(m[660], 111);
      *(heap.alloc(layout.clone()).unwrap().as_ptr() as *mut u8) = 222;
      assert_eq!(m[698], 222);
      *(heap.alloc(layout.clone()).unwrap().as_ptr() as *mut u8) = 123;
      assert_eq!(m[736], 123);
      heap
        .dealloc(NonNull::new_unchecked((o + 660) as *mut u8), layout.clone());
      assert_eq!(m[660], 0);
      heap
        .dealloc(NonNull::new_unchecked((o + 736) as *mut u8), layout.clone());
      assert_eq!(*(&m[736] as *const _ as *const usize), o + 660);
      *(heap.alloc(layout.clone()).unwrap().as_ptr() as *mut u8) = 202;
      assert_eq!(m[736], 202);
      heap
        .dealloc(NonNull::new_unchecked((o + 698) as *mut u8), layout.clone());
      assert_eq!(*(&m[698] as *const _ as *const usize), o + 660);
      heap
        .dealloc(NonNull::new_unchecked((o + 736) as *mut u8), layout.clone());
      assert_eq!(*(&m[736] as *const _ as *const usize), o + 698);
    }
  }
}
