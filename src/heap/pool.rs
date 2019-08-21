use core::{
    alloc::Layout,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering::*},
};

/// The set of free memory blocks.
///
/// It operates by connecting unallocated regions of memory together in a linked
/// list, using the first word of each unallocated region as a pointer to the
/// next.
pub struct Pool {
    /// Free List of previously allocated blocks.
    free: AtomicPtr<u8>,
    /// Growing pointer to the inclusive left edge of the uninitialized area.
    head: AtomicPtr<u8>,
    /// Non-inclusive right edge of the pool.
    edge: *mut u8,
    /// Block size.
    size: usize,
}

impl Pool {
    /// Creates an uninitialized `Pool`.
    ///
    /// The returned pool should be initialized in the run-time with
    /// [`init`](Pool::init) method before use.
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
    /// This method **must** be called before any use of the pool.
    ///
    /// The time complexity of this method is *O(1)*.
    ///
    /// # Safety
    ///
    /// * Calling this method while live allocations exists may lead to data
    ///   corruption.
    pub unsafe fn init(&mut self, start: &mut usize) {
        let offset = start as *mut _ as usize;
        let head = self.head.get_mut();
        *head = head.add(offset);
        self.edge = self.edge.add(offset);
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
        unsafe { self.alloc_free().or_else(|| self.alloc_head()) }
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
