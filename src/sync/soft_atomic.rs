//! Software-implemented atomic types.
//!
//! Atomic types from this module don't require harware support of atomics.
//! They are implemented with [critical sections](Interrupts).

use crate::platform::Interrupts;
use core::cell::UnsafeCell;
use core::{fmt, mem};

mod sealed {
    pub trait AtMostWordSized {}

    impl AtMostWordSized for bool {}
    impl AtMostWordSized for i8 {}
    impl AtMostWordSized for i16 {}
    impl AtMostWordSized for i32 {}
    #[cfg(target_pointer_width = "64")]
    impl AtMostWordSized for i64 {}
    impl AtMostWordSized for isize {}
    impl AtMostWordSized for u8 {}
    impl AtMostWordSized for u16 {}
    impl AtMostWordSized for u32 {}
    #[cfg(target_pointer_width = "64")]
    impl AtMostWordSized for u64 {}
    impl AtMostWordSized for usize {}
    impl<T: ?Sized> AtMostWordSized for *mut T {}
    impl<T: ?Sized> AtMostWordSized for *const T {}
}

/// Software-implemented generic atomic type.
#[derive(Default)]
#[repr(transparent)]
pub struct Atomic<T: sealed::AtMostWordSized + Copy> {
    inner: UnsafeCell<T>,
}

unsafe impl<T: sealed::AtMostWordSized + Copy> Send for Atomic<T> {}
unsafe impl<T: sealed::AtMostWordSized + Copy> Sync for Atomic<T> {}

impl<T: sealed::AtMostWordSized + Copy> Atomic<T> {
    /// Creates a new `Atomic<T>`.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self { inner: UnsafeCell::new(value) }
    }

    /// Consumes the atomic and returns the contained value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    /// Returns a mutable reference to the underlying value.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    /// Returns a mutable pointer to the underlying value.
    #[inline]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.inner.get()
    }

    /// Loads a value from the atomic.
    #[inline]
    pub fn load(&self) -> T {
        unsafe { *self.inner.get() }
    }

    /// Stores a value into the atomic.
    #[inline]
    pub fn store(&self, value: T) {
        unsafe { *self.inner.get() = value };
    }

    /// Stores a value into the atomic, returning the previous value.
    #[inline]
    pub fn swap(&self, value: T) -> T {
        Interrupts::paused(|| unsafe { mem::replace(&mut *self.inner.get(), value) })
    }

    /// Performs read-modify-write sequence, returning the previus value.
    #[inline]
    pub fn modify<F: FnOnce(T) -> T>(&self, f: F) -> T {
        Interrupts::paused(|| {
            let prev = self.load();
            let next = f(prev);
            self.store(next);
            prev
        })
    }

    /// Tries to perform read-modify-write sequence, returning the previus
    /// value.
    #[inline]
    pub fn try_modify<F: FnOnce(T) -> Option<T>>(&self, f: F) -> Result<T, T> {
        Interrupts::paused(|| {
            let prev = self.load();
            if let Some(next) = f(prev) {
                self.store(next);
                Ok(prev)
            } else {
                Err(prev)
            }
        })
    }
}

impl<T: sealed::AtMostWordSized + Copy> fmt::Debug for Atomic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Atomic").finish_non_exhaustive()
    }
}
