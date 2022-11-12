//! Basic functions for dealing with memory.

use crate::platform::{data_mem_init, zeroed_mem_init};
use core::cell::UnsafeCell;

extern "C" {
    static BSS_BASE: UnsafeCell<usize>;
    static BSS_END: UnsafeCell<usize>;
    static DATA_LOAD: UnsafeCell<usize>;
    static DATA_BASE: UnsafeCell<usize>;
    static DATA_END: UnsafeCell<usize>;
}

/// Initializes global mutable memory.
///
/// This function **must** be called as early as possible, because it
/// initializes `static` variables.
///
/// # Safety
///
/// This function must be called only once and before using any global
/// variables.
pub unsafe fn init() {
    unsafe {
        zeroed_mem_init(&BSS_BASE, &BSS_END);
        data_mem_init(&DATA_LOAD, &DATA_BASE, &DATA_END);
    }
}
