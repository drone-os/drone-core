//! Basic functions for dealing with memory.

use crate::platform::{data_section_init, zeroed_section_init};
use core::cell::UnsafeCell;

extern "C" {
    static BSS_BASE: UnsafeCell<usize>;
    static BSS_END: UnsafeCell<usize>;
    static DATA_LOAD: UnsafeCell<usize>;
    static DATA_BASE: UnsafeCell<usize>;
    static DATA_END: UnsafeCell<usize>;
}

/// Initializes the `BSS` mutable memory section.
///
/// This function **must** be called as early as possible.
///
/// See also [`data_init`].
///
/// # Safety
///
/// This function reverts the state of initially zeroed mutable statics.
pub unsafe fn bss_init() {
    unsafe { zeroed_section_init(&BSS_BASE, &BSS_END) };
}

/// Initializes the `DATA` mutable memory section.
///
/// This function **must** be called as early as possible.
///
/// See also [`bss_init`].
///
/// # Safety
///
/// This function reverts the state of initially non-zeroed mutable statics.
pub unsafe fn data_init() {
    unsafe { data_section_init(&DATA_LOAD, &DATA_BASE, &DATA_END) };
}
