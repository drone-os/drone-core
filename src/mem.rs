//! Basic functions for dealing with memory.

use core::{cell::UnsafeCell, ptr};

extern "C" {
    static BSS_START: UnsafeCell<usize>;
    static BSS_END: UnsafeCell<usize>;
    static DATA_LOAD: UnsafeCell<usize>;
    static DATA_START: UnsafeCell<usize>;
    static DATA_END: UnsafeCell<usize>;
}

/// Initializes the BSS mutable memory segment.
///
/// This function **must** be called as early as possible.
///
/// See also [`data_init`].
///
/// # Safety
///
/// This function reverts the state of initially zeroed mutable statics.
pub unsafe fn bss_init() {
    unsafe {
        let length = BSS_END.get() as usize - BSS_START.get() as usize;
        ptr::write_bytes(BSS_START.get(), 0, length >> 2);
    }
}

/// Initializes the DATA mutable memory segment.
///
/// This function **must** be called as early as possible.
///
/// See also [`bss_init`].
///
/// # Safety
///
/// This function reverts the state of initially non-zeroed mutable statics.
pub unsafe fn data_init() {
    unsafe {
        let length = DATA_END.get() as usize - DATA_START.get() as usize;
        ptr::copy_nonoverlapping(DATA_LOAD.get(), DATA_START.get(), length >> 2);
    }
}
