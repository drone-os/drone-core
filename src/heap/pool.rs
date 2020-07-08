use core::{
    alloc::Layout,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

/// The set of free memory blocks.
///
/// It operates by connecting unallocated regions of memory together in a linked
/// list, using the first word of each unallocated region as a pointer to the
/// next.
pub struct Pool {
    /// Block size. Doesn't change in the run-time.
    size: usize,
    /// Address of the byte past the last element. Doesn't change in the
    /// run-time.
    edge: *mut u8,
    /// Free List of previously allocated blocks.
    free: AtomicPtr<u8>,
    /// Pointer growing from the starting address until it reaches the `edge`.
    uninit: AtomicPtr<u8>,
}

unsafe impl Sync for Pool {}

impl Pool {
    /// Creates a new `Pool`.
    pub const fn new(address: usize, size: usize, capacity: usize) -> Self {
        Self {
            size,
            edge: (address + size * capacity) as *mut u8,
            free: AtomicPtr::new(ptr::null_mut()),
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
    pub fn alloc(&self) -> Option<NonNull<u8>> {
        unsafe { self.alloc_free().or_else(|| self.alloc_uninit()) }
    }

    /// Deallocates the block referenced by `ptr`.
    ///
    /// This operation is lock-free and has *O(1)* time complexity.
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a block previously allocated by
    ///   [`alloc`](Pool::alloc).
    /// * `ptr` must not be used after deallocation.
    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn dealloc(&self, ptr: NonNull<u8>) {
        loop {
            let curr = self.free.load(Ordering::Acquire);
            unsafe { ptr::write(ptr.as_ptr() as *mut *mut u8, curr) };
            let next = ptr.as_ptr() as *mut u8;
            if self.free.compare_and_swap(curr, next, Ordering::AcqRel) == curr {
                break;
            }
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn alloc_free(&self) -> Option<NonNull<u8>> {
        loop {
            let curr = self.free.load(Ordering::Acquire);
            if curr.is_null() {
                break None;
            }
            let next = unsafe { ptr::read(curr as *const *mut u8) };
            if self.free.compare_and_swap(curr, next, Ordering::AcqRel) == curr {
                break Some(unsafe { NonNull::new_unchecked(curr) });
            }
        }
    }

    unsafe fn alloc_uninit(&self) -> Option<NonNull<u8>> {
        loop {
            let curr = self.uninit.load(Ordering::Relaxed);
            if curr == self.edge {
                break None;
            }
            let next = unsafe { curr.add(self.size) };
            if self.uninit.compare_and_swap(curr, next, Ordering::Relaxed) == curr {
                break Some(unsafe { NonNull::new_unchecked(curr) });
            }
        }
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
        (self.as_ptr() as *mut u8) < pool.edge
    }
}
