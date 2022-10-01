use core::alloc::Layout;
use core::ptr::{self, NonNull};

#[cfg(all(feature = "atomics", not(loom)))]
type AtomicPtr = core::sync::atomic::AtomicPtr<u8>;
#[cfg(all(feature = "atomics", loom))]
type AtomicPtr = loom::sync::atomic::AtomicPtr<u8>;
#[cfg(not(feature = "atomics"))]
type AtomicPtr = crate::sync::soft_atomic::Atomic<*mut u8>;

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
    /// Free List of previously allocated blocks.
    free: AtomicPtr,
    /// Pointer growing from the starting address until it reaches the `edge`.
    uninit: AtomicPtr,
}

unsafe impl Sync for Pool {}

macro_rules! maybe_const_fn {
    ($(#[$($attr:tt)*])* $vis:vis const fn $name:ident($($args:tt)*) -> $ret:ty { $($body:tt)* }) => {
        #[cfg(not(loom))]
        $(#[$($attr)*])* $vis const fn $name($($args)*) -> $ret { $($body)* }
        #[cfg(loom)]
        $(#[$($attr)*])* $vis fn $name($($args)*) -> $ret { $($body)* }
    };
}

impl Pool {
    maybe_const_fn! {
        /// Creates a new `Pool`.
        pub const fn new(address: usize, size: usize, count: usize) -> Self {
            Self {
                size,
                edge: (address + size * count) as *mut u8,
                free: AtomicPtr::new(ptr::null_mut()),
                uninit: AtomicPtr::new(address as *mut u8),
            }
        }
    }

    /// Returns the block size.
    pub const fn size(&self) -> usize {
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
        self.allocate_free()
            .or_else(|| self.allocate_uninit())
            .map(|ptr| unsafe { NonNull::new_unchecked(ptr) })
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
    pub unsafe fn deallocate(&self, ptr: NonNull<u8>) {
        load_modify_atomic!(self.free, Acquire, AcqRel, |curr| unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            ptr.as_ptr().cast::<*mut u8>().write(curr);
            ptr.as_ptr().cast::<u8>()
        });
    }

    fn allocate_free(&self) -> Option<*mut u8> {
        load_try_modify_atomic!(self.free, Acquire, AcqRel, |curr| unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            (!curr.is_null()).then(|| (curr as *const *mut u8).read())
        })
        .ok()
    }

    fn allocate_uninit(&self) -> Option<*mut u8> {
        load_try_modify_atomic!(self.uninit, Relaxed, Relaxed, |curr| unsafe {
            (curr != self.edge).then(|| curr.add(self.size))
        })
        .ok()
    }
}

pub trait Fits: Copy {
    fn fits(self, pool: &Pool) -> bool;
}

impl<'a> Fits for &'a Layout {
    fn fits(self, pool: &Pool) -> bool {
        self.size() <= pool.size
    }
}

impl Fits for NonNull<u8> {
    fn fits(self, pool: &Pool) -> bool {
        self.as_ptr().cast::<u8>() < pool.edge
    }
}
