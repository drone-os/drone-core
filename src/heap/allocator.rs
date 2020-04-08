use super::pool::{Fits, Pool};
use core::{
    alloc::{AllocErr, Layout},
    ptr::NonNull,
    slice::SliceIndex,
};

/// Allocator for a generic memory pools layout.
///
/// The trait is supposed to be implemented for an array of pools.
/// [`heap`](crate::heap) macro should be used to generate the concrete type and
/// the implementation.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub trait Allocator: Sized {
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
}

/// Does a binary search for the pool with the smallest block size to fit
/// `value`.
pub fn binary_search<A: Allocator, T: Fits>(heap: &A, value: T) -> usize {
    let (mut left, mut right) = (0, A::POOL_COUNT);
    while right > left {
        let middle = left + ((right - left) >> 1);
        let pool = unsafe { heap.get_pool_unchecked(middle) };
        if value.fits(pool) {
            right = middle;
        } else {
            left = middle + 1;
        }
    }
    left
}

#[doc(hidden)]
pub fn alloc<A: Allocator>(heap: &A, layout: Layout) -> Result<(NonNull<u8>, usize), AllocErr> {
    for pool_idx in binary_search(heap, &layout)..A::POOL_COUNT {
        let pool = unsafe { heap.get_pool_unchecked(pool_idx) };
        if let Some(ptr) = pool.alloc() {
            #[cfg(feature = "heaptrace")]
            trace::alloc(layout);
            return Ok((ptr, pool.size()));
        }
    }
    Err(AllocErr)
}

#[allow(clippy::used_underscore_binding)]
#[doc(hidden)]
pub unsafe fn dealloc<A: Allocator>(heap: &A, ptr: NonNull<u8>, _layout: Layout) {
    let pool = heap.get_pool_unchecked(binary_search(heap, ptr));
    #[cfg(feature = "heaptrace")]
    trace::dealloc(_layout);
    pool.dealloc(ptr);
}

#[cfg(feature = "heaptrace")]
mod trace {
    use crate::{
        heap::HEAPTRACE_KEY,
        log::{Port, HEAPTRACE_PORT},
    };
    use core::{alloc::Layout, slice};

    #[inline(always)]
    pub(super) fn alloc(layout: Layout) {
        #[inline(never)]
        fn trace(layout: Layout) {
            #[cfg(feature = "std")]
            return unimplemented!();
            let packet = [0xABCD_u32.to_be(), (layout.size() as u32) ^ HEAPTRACE_KEY];
            Port::exclusive(HEAPTRACE_PORT).write_bytes(unsafe {
                slice::from_raw_parts((packet.as_ptr() as *const u8).add(2), 6)
            });
        }
        if Port::new(HEAPTRACE_PORT).is_enabled() {
            trace(layout);
        }
    }

    #[inline(always)]
    pub(super) fn dealloc(layout: Layout) {
        #[inline(never)]
        fn trace(layout: Layout) {
            #[cfg(feature = "std")]
            return unimplemented!();
            let packet = [0xDCBA_u32.to_be(), (layout.size() as u32) ^ HEAPTRACE_KEY];
            Port::exclusive(HEAPTRACE_PORT).write_bytes(unsafe {
                slice::from_raw_parts((packet.as_ptr() as *const u8).add(2), 6)
            });
        }
        if Port::new(HEAPTRACE_PORT).is_enabled() {
            trace(layout);
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
    }

    #[test]
    fn test_binary_search() {
        fn search_layout(heap: &TestHeap, size: usize) -> Option<usize> {
            let pool_idx = binary_search(heap, &Layout::from_size_align(size, 4).unwrap());
            if pool_idx < TestHeap::POOL_COUNT {
                unsafe { Some(heap.get_pool_unchecked(pool_idx).size()) }
            } else {
                None
            }
        }
        fn search_ptr(heap: &TestHeap, ptr: usize) -> Option<usize> {
            let pool_idx = binary_search(heap, unsafe { NonNull::new_unchecked(ptr as *mut u8) });
            if pool_idx < TestHeap::POOL_COUNT {
                unsafe { Some(heap.get_pool_unchecked(pool_idx).size()) }
            } else {
                None
            }
        }
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
        assert_eq!(search_layout(&heap, 1), Some(2));
        assert_eq!(search_layout(&heap, 2), Some(2));
        assert_eq!(search_layout(&heap, 15), Some(16));
        assert_eq!(search_layout(&heap, 16), Some(16));
        assert_eq!(search_layout(&heap, 17), Some(23));
        assert_eq!(search_layout(&heap, 91), Some(91));
        assert_eq!(search_layout(&heap, 92), None);
        assert_eq!(search_ptr(&heap, 0), Some(2));
        assert_eq!(search_ptr(&heap, 20), Some(2));
        assert_eq!(search_ptr(&heap, 219), Some(2));
        assert_eq!(search_ptr(&heap, 220), Some(5));
        assert_eq!(search_ptr(&heap, 719), Some(5));
        assert_eq!(search_ptr(&heap, 720), Some(8));
        assert_eq!(search_ptr(&heap, 721), Some(8));
        assert_eq!(search_ptr(&heap, 5000), Some(23));
        assert_eq!(search_ptr(&heap, 23220), Some(91));
        assert_eq!(search_ptr(&heap, 32319), Some(91));
        assert_eq!(search_ptr(&heap, 32320), None);
        assert_eq!(search_ptr(&heap, 50000), None);
    }

    #[test]
    fn allocations() {
        unsafe fn alloc_and_set(heap: &TestHeap, layout: Layout, value: u8) {
            *(alloc(heap, layout).unwrap().0.as_ptr() as *mut u8) = value;
        }
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
            alloc_and_set(&heap, layout, 111);
            assert_eq!(m[660], 111);
            alloc_and_set(&heap, layout, 222);
            assert_eq!(m[698], 222);
            alloc_and_set(&heap, layout, 123);
            assert_eq!(m[736], 123);
            dealloc(&heap, NonNull::new_unchecked((o + 660) as *mut u8), layout);
            assert_eq!(m[660], 0);
            dealloc(&heap, NonNull::new_unchecked((o + 736) as *mut u8), layout);
            assert_eq!(*(&m[736] as *const _ as *const usize), o + 660);
            alloc_and_set(&heap, layout, 202);
            assert_eq!(m[736], 202);
            dealloc(&heap, NonNull::new_unchecked((o + 698) as *mut u8), layout);
            assert_eq!(*(&m[698] as *const _ as *const usize), o + 660);
            dealloc(&heap, NonNull::new_unchecked((o + 736) as *mut u8), layout);
            assert_eq!(*(&m[736] as *const _ as *const usize), o + 698);
        }
    }
}
