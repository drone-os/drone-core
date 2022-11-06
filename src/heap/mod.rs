//! Dynamic memory allocation.
//!
//! Dynamic memory is crucial for Drone operation. Objectives like real-time
//! characteristics, high concurrency, small code size, fast execution have led
//! to Memory Pools design of the heap. All operations are lock-free and have
//! *O(1)* time complexity, which means they are deterministic.
//!
//! The continuous memory region for the heap is split into pools. A pool is
//! further split into fixed-sized blocks that hold actual allocations. A pool
//! is defined by its block-size and the number of blocks. The pools
//! configuration should be defined in the compile-time. A drawback of this
//! approach is that memory pools may need to be tuned for the application.
//!
//! # Usage
//!
//! Add the heap configuration to the `layout.toml`:
//!
//! ```toml
//! [heap]
//! size = "10K"
//! pools = [
//!     { block = "4", count = "896" },
//!     { block = "32", count = "80" },
//!     { block = "256", count = "16" },
//! ]
//! ```
//!
//! The `size` field should match the resulting size of the pools.
//!
//! Then in the application code:
//!
//! ```no_run
//! # #![feature(allocator_api)]
//! # #![feature(slice_ptr_get)]
//! # drone_core::override_layout! { r#"
//! # [ram]
//! # main = { origin = 0x20000000, size = "20K" }
//! # [data]
//! # ram = "main"
//! # [heap.main]
//! # ram = "main"
//! # size = "10K"
//! # pools = [
//! #     { block = "4", count = "896" },
//! #     { block = "32", count = "80" },
//! #     { block = "256", count = "16" },
//! # ]
//! # "# }
//! # fn main() {}
//! use drone_core::heap;
//!
//! // Define a concrete heap type with the layout defined in the layout.toml
//! heap! {
//!     // Heap name in `layout.toml`.
//!     layout => main;
//!     /// The main heap allocator generated from the `layout.toml`.
//!     metadata => pub Heap;
//!     /// The global allocator.
//!     #[global_allocator] // Use this heap as the global allocator.
//!     instance => pub HEAP;
//!     // Uncomment the following line to enable heap tracing feature:
//!     // enable_trace_stream => 31;
//! }
//! ```
//!
//! # Tuning
//!
//! Using empiric values for the memory pools layout may lead to undesired
//! memory fragmentation. Eventually the layout will need to be tuned for the
//! application. Drone can capture allocation statistics from the real target
//! device at the run-time and generate an optimized memory layout for this
//! specific application. Ideally this will result in zero fragmentation.
//!
//! The actual steps are platform-specific. Refer to the platform crate
//! documentation for instructions.

mod pool;
#[doc(hidden)]
pub mod trace;

pub use self::pool::Pool;
use self::pool::{pool_by_ptr, pool_range_by_layout};
use core::alloc::{AllocError, Layout};
use core::ptr;
use core::ptr::NonNull;

#[doc(hidden)]
#[inline(never)]
#[export_name = "heap_allocate"]
pub fn allocate(pools: &[Pool], layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
    if layout.size() == 0 {
        return Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0));
    }
    for i in pool_range_by_layout(pools, &layout) {
        let pool = unsafe { pools.get_unchecked(i) };
        if let Some(ptr) = pool.allocate() {
            return Ok(NonNull::slice_from_raw_parts(ptr, pool.size()));
        }
    }
    Err(AllocError)
}

#[doc(hidden)]
#[inline(never)]
#[export_name = "heap_allocate_zeroed"]
pub fn allocate_zeroed(pools: &[Pool], layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
    let ptr = allocate(pools, layout)?;
    unsafe { ptr.as_non_null_ptr().as_ptr().write_bytes(0, ptr.len()) }
    Ok(ptr)
}

#[doc(hidden)]
#[inline(never)]
#[export_name = "heap_deallocate"]
pub unsafe fn deallocate(pools: &[Pool], base: *mut u8, ptr: NonNull<u8>, layout: Layout) {
    if layout.size() == 0 {
        return;
    }
    if let Some(i) = pool_by_ptr(pools, base, ptr) {
        unsafe { pools.get_unchecked(i).deallocate(ptr) };
    }
}

#[doc(hidden)]
#[inline(never)]
#[export_name = "heap_grow"]
pub unsafe fn grow(
    pools: &[Pool],
    base: *mut u8,
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_layout: Layout,
) -> Result<NonNull<[u8]>, AllocError> {
    unsafe {
        let new_ptr = allocate(pools, new_layout)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_layout.size());
        deallocate(pools, base, ptr, old_layout);
        Ok(new_ptr)
    }
}

#[doc(hidden)]
#[inline(never)]
#[export_name = "heap_grow_zeroed"]
pub unsafe fn grow_zeroed(
    pools: &[Pool],
    base: *mut u8,
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_layout: Layout,
) -> Result<NonNull<[u8]>, AllocError> {
    unsafe {
        let new_ptr = allocate_zeroed(pools, new_layout)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_layout.size());
        deallocate(pools, base, ptr, old_layout);
        Ok(new_ptr)
    }
}

#[doc(hidden)]
#[inline(never)]
#[export_name = "heap_shrink"]
pub unsafe fn shrink(
    pools: &[Pool],
    base: *mut u8,
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_layout: Layout,
) -> Result<NonNull<[u8]>, AllocError> {
    unsafe {
        let new_ptr = allocate(pools, new_layout)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), new_layout.size());
        deallocate(pools, base, ptr, old_layout);
        Ok(new_ptr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHeap {
        base: *mut u8,
        pools: [Pool; 10],
    }

    #[test]
    fn test_binary_search() {
        fn search_layout(heap: &TestHeap, size: usize) -> Option<usize> {
            let pool_range =
                pool_range_by_layout(&heap.pools, &Layout::from_size_align(size, 4).unwrap());
            let pool_idx = pool_range.start;
            if pool_idx < heap.pools.len() {
                unsafe { Some(heap.pools.get_unchecked(pool_idx).size()) }
            } else {
                None
            }
        }
        fn search_ptr(heap: &TestHeap, ptr: usize) -> Option<usize> {
            let pool_idx = pool_by_ptr(&heap.pools, heap.base, unsafe {
                NonNull::new_unchecked(ptr as *mut u8)
            })?;
            if pool_idx < heap.pools.len() {
                unsafe { Some(heap.pools.get_unchecked(pool_idx).size()) }
            } else {
                None
            }
        }
        let heap = TestHeap {
            base: 20 as *mut u8,
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
        assert_eq!(search_ptr(&heap, 0), None);
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
        #[track_caller]
        unsafe fn allocate_and_set(heap: &TestHeap, layout: Layout, value: u8) {
            unsafe {
                *allocate(&heap.pools, layout).unwrap().as_mut_ptr() = value;
            }
        }
        #[track_caller]
        unsafe fn dealloc(heap: &TestHeap, layout: Layout, address: usize) {
            unsafe {
                deallocate(
                    &heap.pools,
                    heap.base,
                    NonNull::new_unchecked(address as *mut u8),
                    layout,
                );
            }
        }
        let mut m = [0u8; 3230];
        let o = &mut m as *mut _ as usize;
        let heap = TestHeap {
            base: o as *mut u8,
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
            allocate_and_set(&heap, layout, 111);
            assert_eq!(m[660], 111);
            allocate_and_set(&heap, layout, 222);
            assert_eq!(m[698], 222);
            allocate_and_set(&heap, layout, 123);
            assert_eq!(m[736], 123);
            dealloc(&heap, layout, o + 660);
            assert_eq!(m[660], 0);
            dealloc(&heap, layout, o + 736);
            assert_eq!(*(&m[736] as *const _ as *const usize), o + 660);
            allocate_and_set(&heap, layout, 202);
            assert_eq!(m[736], 202);
            dealloc(&heap, layout, o + 698);
            assert_eq!(*(&m[698] as *const _ as *const usize), o + 660);
            dealloc(&heap, layout, o + 736);
            assert_eq!(*(&m[736] as *const _ as *const usize), o + 698);
        }
    }
}
