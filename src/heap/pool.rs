#[cfg(not(feature = "atomics"))]
use crate::sync::soft_atomic::Atomic;
#[cfg(feature = "atomics")]
use core::sync::atomic::{AtomicPtr, Ordering};
use core::{alloc::Layout, ptr, ptr::NonNull};

/// The set of free memory blocks.
///
/// It operates by connecting unallocated regions of memory together in a linked
/// list, using the first word of each unallocated region as a pointer to the
/// next.
// This structure should be kept in sync with drone-ld.
#[repr(C)]
pub struct Pool {
    /// Block size. This field is immutable.
    size: usize,
    /// Address of the byte past the last element. This field is immutable.
    edge: *mut u8,
    #[cfg(not(feature = "atomics"))]
    /// Free List of previously allocated blocks.
    free: Atomic<*mut u8>,
    #[cfg(feature = "atomics")]
    /// Free List of previously allocated blocks.
    free: AtomicPtr<u8>,
    #[cfg(not(feature = "atomics"))]
    /// Pointer growing from the starting address until it reaches the `edge`.
    uninit: Atomic<*mut u8>,
    #[cfg(feature = "atomics")]
    /// Pointer growing from the starting address until it reaches the `edge`.
    uninit: AtomicPtr<u8>,
}

unsafe impl Sync for Pool {}

impl Pool {
    /// Creates a new `Pool`.
    pub const fn new(address: usize, size: usize, count: usize) -> Self {
        Self {
            size,
            edge: (address + size * count) as *mut u8,
            #[cfg(not(feature = "atomics"))]
            free: Atomic::new(ptr::null_mut()),
            #[cfg(feature = "atomics")]
            free: AtomicPtr::new(ptr::null_mut()),
            #[cfg(not(feature = "atomics"))]
            uninit: Atomic::new(address as *mut u8),
            #[cfg(feature = "atomics")]
            uninit: AtomicPtr::new(address as *mut u8),
        }
    }

    /// Returns the block size.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Allocates one block of memory.
    ///
    /// If this method returns `Some(addr)`, then the `addr` returned will be
    /// non-null address pointing to the block. If this method returns `None`,
    /// then the pool is exhausted.
    ///
    /// This operation is lock-free and has *O(1)* time complexity.
    pub fn allocate(&self) -> Option<NonNull<u8>> {
        unsafe { self.alloc_free().or_else(|| self.alloc_uninit()) }
    }

    /// Deallocates the block referenced by `ptr`.
    ///
    /// This operation is lock-free and has *O(1)* time complexity.
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a block previously allocated by
    ///   [`allocate`](Pool::allocate).
    /// * `ptr` must not be used after deallocation.
    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn deallocate(&self, ptr: NonNull<u8>) {
        let modify = |curr| {
            unsafe { ptr::write(ptr.as_ptr().cast::<*mut u8>(), curr) };
            ptr.as_ptr().cast::<u8>()
        };
        #[cfg(not(feature = "atomics"))]
        self.free.modify(modify);
        #[cfg(feature = "atomics")]
        loop {
            let curr = self.free.load(Ordering::Acquire);
            if self
                .free
                .compare_exchange_weak(curr, modify(curr), Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn alloc_free(&self) -> Option<NonNull<u8>> {
        let modify =
            |curr: *mut u8| (!curr.is_null()).then(|| unsafe { ptr::read(curr as *const *mut u8) });
        #[cfg(not(feature = "atomics"))]
        let ptr = self.free.try_modify(modify).ok();
        #[cfg(feature = "atomics")]
        let ptr = loop {
            let curr = self.free.load(Ordering::Acquire);
            if let Some(next) = modify(curr) {
                if self
                    .free
                    .compare_exchange_weak(curr, next, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    break Some(curr);
                }
            } else {
                break None;
            }
        };
        ptr.map(|curr| unsafe { NonNull::new_unchecked(curr) })
    }

    unsafe fn alloc_uninit(&self) -> Option<NonNull<u8>> {
        let modify = |curr: *mut u8| (curr != self.edge).then(|| unsafe { curr.add(self.size) });
        #[cfg(not(feature = "atomics"))]
        let ptr = self.free.try_modify(modify).ok();
        #[cfg(feature = "atomics")]
        let ptr = loop {
            let curr = self.uninit.load(Ordering::Relaxed);
            if let Some(next) = modify(curr) {
                if self
                    .uninit
                    .compare_exchange_weak(curr, next, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    break Some(curr);
                }
            } else {
                break None;
            }
        };
        ptr.map(|curr| unsafe { NonNull::new_unchecked(curr) })
    }
}

pub trait Fits: Copy {
    fn fits(self, pool: &Pool) -> bool;
}

impl<'a> Fits for &'a Layout {
    #[inline]
    fn fits(self, pool: &Pool) -> bool {
        self.size() <= pool.size
    }
}

impl Fits for NonNull<u8> {
    #[inline]
    fn fits(self, pool: &Pool) -> bool {
        (self.as_ptr().cast::<u8>()) < pool.edge
    }
}
