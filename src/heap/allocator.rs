use super::pool::{Fits, Pool};
use core::{
    alloc::{AllocErr, CannotReallocInPlace, Excess, Layout},
    cmp,
    ptr::{self, NonNull},
    slice::SliceIndex,
};

/// Allocator for a generic memory pools layout.
///
/// The trait is supposed to be implemented for an array of pools.
/// [`heap`](crate::heap) macro should be used to generate the concrete type and
/// the implementation.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub trait Allocator {
    /// The total number of memory pools.
    const POOL_COUNT: usize;

    /// Returns a reference to a pool or subslice, without doing bounds
    /// checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is Undefined Behavior.
    unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
    where
        I: SliceIndex<[Pool]>;

    /// Returns a mutable reference to a pool or subslice, without doing bounds
    /// checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is Undefined Behavior.
    unsafe fn get_pool_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
    where
        I: SliceIndex<[Pool]>;

    /// Empty allocation hook. Can be re-defined by the implementation.
    #[inline]
    fn alloc_hook(_layout: Layout, _pool: &Pool) {}

    /// Empty deallocation hook. Can be re-defined by the implementation.
    #[inline]
    fn dealloc_hook(_layout: Layout, _pool: &Pool) {}

    /// Empty growing in place hook. Can be re-defined by the implementation.
    #[inline]
    fn grow_in_place_hook(_layout: Layout, _new_size: usize) {}

    /// Empty shrinking in place hook. Can be re-defined by the implementation.
    #[inline]
    fn shrink_in_place_hook(_layout: Layout, _new_size: usize) {}

    /// Does a binary search for the pool with the smallest block size to fit
    /// `value`.
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
    unsafe fn alloc<'a, T: WithPool<'a, U>, U: MaybePool<'a>>(
        &'a self,
        layout: Layout,
    ) -> Result<T, AllocErr> {
        for pool_idx in self.binary_search(&layout)..Self::POOL_COUNT {
            let pool = self.get_pool_unchecked(pool_idx);
            if let Some(ptr) = pool.alloc() {
                Self::alloc_hook(layout, pool);
                return Ok(T::from(ptr, || U::from(pool)));
            }
        }
        Err(AllocErr)
    }

    #[doc(hidden)]
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn realloc<'a, T: WithPool<'a, U>, U: MaybePool<'a>>(
        &'a self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<T, AllocErr> {
        if new_size < layout.size() {
            let _ = self.shrink_in_place(ptr, layout, new_size);
            Ok(T::from(ptr, || U::from(self.get_pool_unchecked(self.binary_search(ptr)))))
        } else if let Ok(pool) = self.grow_in_place(ptr, layout, new_size) {
            Ok(T::from(ptr, || pool))
        } else {
            self.alloc(Layout::from_size_align_unchecked(new_size, layout.align())).map(
                |new_ptr: T| {
                    ptr::copy_nonoverlapping(
                        ptr.as_ptr() as *const usize,
                        new_ptr.as_ptr() as *mut usize,
                        cmp::min(layout.size(), new_size),
                    );
                    self.dealloc(ptr, layout);
                    new_ptr
                },
            )
        }
    }

    #[doc(hidden)]
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        let pool = self.get_pool_unchecked(self.binary_search(ptr));
        Self::dealloc_hook(layout, pool);
        pool.dealloc(ptr);
    }

    #[doc(hidden)]
    unsafe fn usable_size(&self, layout: &Layout) -> (usize, usize) {
        let pool = self.get_pool_unchecked(self.binary_search(layout));
        (0, pool.size())
    }

    #[doc(hidden)]
    unsafe fn grow_in_place<'a, T: MaybePool<'a>>(
        &'a self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<T, CannotReallocInPlace> {
        let pool = self.get_pool_unchecked(self.binary_search(ptr));
        if Layout::from_size_align_unchecked(new_size, layout.align()).fits(pool) {
            Self::grow_in_place_hook(layout, new_size);
            Ok(T::from(pool))
        } else {
            Err(CannotReallocInPlace)
        }
    }

    #[doc(hidden)]
    unsafe fn shrink_in_place(
        &self,
        _ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<(), CannotReallocInPlace> {
        Self::shrink_in_place_hook(layout, new_size);
        Ok(())
    }
}

pub trait MaybePool<'a> {
    fn from(pool: &'a Pool) -> Self;
}

impl<'a> MaybePool<'a> for () {
    #[inline]
    fn from(_pool: &'a Pool) {}
}

impl<'a> MaybePool<'a> for &'a Pool {
    #[inline]
    fn from(pool: Self) -> Self {
        pool
    }
}

pub trait WithPool<'a, T: MaybePool<'a>> {
    fn from<F>(ptr: NonNull<u8>, pool: F) -> Self
    where
        F: FnOnce() -> T;

    fn as_ptr(&self) -> *mut u8;
}

impl<'a> WithPool<'a, ()> for NonNull<u8> {
    #[inline]
    fn from<F>(ptr: NonNull<u8>, _pool: F) -> Self
    where
        F: FnOnce(),
    {
        ptr
    }

    #[inline]
    fn as_ptr(&self) -> *mut u8 {
        NonNull::as_ptr(*self)
    }
}

impl<'a> WithPool<'a, &'a Pool> for (NonNull<u8>, &'a Pool) {
    #[inline]
    fn from<F>(ptr: NonNull<u8>, pool: F) -> Self
    where
        F: FnOnce() -> &'a Pool,
    {
        (ptr, pool())
    }

    #[inline]
    fn as_ptr(&self) -> *mut u8 {
        NonNull::as_ptr(self.0)
    }
}

impl<'a> WithPool<'a, &'a Pool> for Excess {
    #[inline]
    fn from<F>(ptr: NonNull<u8>, pool: F) -> Self
    where
        F: FnOnce() -> &'a Pool,
    {
        Self(ptr, pool().size())
    }

    #[inline]
    fn as_ptr(&self) -> *mut u8 {
        NonNull::as_ptr(self.0)
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

    impl TestHeap {
        fn search_layout(&self, size: usize) -> Option<usize> {
            let pool_idx = self.binary_search(&Layout::from_size_align(size, 4).unwrap());
            if pool_idx < TestHeap::POOL_COUNT {
                unsafe { Some(self.get_pool_unchecked(pool_idx).size()) }
            } else {
                None
            }
        }

        fn search_ptr(&self, ptr: usize) -> Option<usize> {
            let pool_idx = self.binary_search(unsafe { NonNull::new_unchecked(ptr as *mut u8) });
            if pool_idx < TestHeap::POOL_COUNT {
                unsafe { Some(self.get_pool_unchecked(pool_idx).size()) }
            } else {
                None
            }
        }

        unsafe fn alloc_and_set(&self, layout: Layout, value: u8) {
            *(self.alloc::<NonNull<u8>, ()>(layout).unwrap().as_ptr() as *mut u8) = value;
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
        let mut m = [0u8; 3230];
        let o = &mut m as *mut _ as usize;
        let heap = TestHeap {
            pools: [
                Pool::new(o + 0, 2, 10),
                Pool::new(o + 20, 5, 10),
                Pool::new(o + 70, 8, 10),
                Pool::new(o + 150, 12, 10),
                Pool::new(o + 270, 16, 10),
                Pool::new(o + 430, 23, 10),
                Pool::new(o + 660, 38, 10),
                Pool::new(o + 1040, 56, 10),
                Pool::new(o + 1600, 72, 10),
                Pool::new(o + 2320, 91, 10),
            ],
        };
        let layout = Layout::from_size_align(32, 1).unwrap();
        unsafe {
            heap.alloc_and_set(layout, 111);
            assert_eq!(m[660], 111);
            heap.alloc_and_set(layout, 222);
            assert_eq!(m[698], 222);
            heap.alloc_and_set(layout, 123);
            assert_eq!(m[736], 123);
            heap.dealloc(NonNull::new_unchecked((o + 660) as *mut u8), layout);
            assert_eq!(m[660], 0);
            heap.dealloc(NonNull::new_unchecked((o + 736) as *mut u8), layout);
            assert_eq!(*(&m[736] as *const _ as *const usize), o + 660);
            heap.alloc_and_set(layout, 202);
            assert_eq!(m[736], 202);
            heap.dealloc(NonNull::new_unchecked((o + 698) as *mut u8), layout);
            assert_eq!(*(&m[698] as *const _ as *const usize), o + 660);
            heap.dealloc(NonNull::new_unchecked((o + 736) as *mut u8), layout);
            assert_eq!(*(&m[736] as *const _ as *const usize), o + 698);
        }
    }
}
