//! Basic functions for dealing with memory.

use core::ptr;

/// Initializes the memory segments.
///
/// This macro **must** be called as early as possible.
///
/// # Examples
///
/// ```no_run
/// use drone_core::mem;
///
/// fn main() {
///     unsafe {
///         mem::init!();
///     }
/// }
/// ```
///
/// # Safety
///
/// Calling this macro reverts the state of mutable statics.
#[doc(inline)]
pub use crate::mem_init as init;

#[doc(hidden)]
#[macro_export]
macro_rules! mem_init {
    () => {{
        extern "C" {
            static BSS_START: ::core::cell::UnsafeCell<usize>;
            static BSS_END: ::core::cell::UnsafeCell<usize>;
            static DATA_CONST: ::core::cell::UnsafeCell<usize>;
            static DATA_START: ::core::cell::UnsafeCell<usize>;
            static DATA_END: ::core::cell::UnsafeCell<usize>;
        }
        $crate::mem::bss_init(BSS_START.get(), BSS_END.get());
        $crate::mem::data_init(DATA_CONST.get(), DATA_START.get(), DATA_END.get());
    }};
}

#[doc(hidden)]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub unsafe fn bss_init(start: *mut usize, end: *const usize) {
    let count = end as usize - start as usize;
    ptr::write_bytes(start, 0, count >> 2);
}

#[doc(hidden)]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub unsafe fn data_init(data: *const usize, start: *mut usize, end: *const usize) {
    let count = end as usize - start as usize;
    ptr::copy_nonoverlapping(data, start, count >> 2);
}
