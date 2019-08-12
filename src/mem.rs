//! Basic functions for dealing with memory.

use core::ptr;

/// Initializes initially zeroed mutable `static`s.
///
/// This function **must** be called before any use of initially zeroed mutable
/// statics.
///
/// See also [`data_init`].
///
/// # Safety
///
/// * Calling this function after mutating initially zeroed statics effectively
///   zeroes them again.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub unsafe fn bss_init(start: &mut usize, end: &usize) {
    let start = start as *mut _;
    let end = end as *const _;
    let count = end as usize - start as usize;
    ptr::write_bytes(start, 0, count >> 2);
}

/// Initializes mutable `static`s.
///
/// This function **must** be called before any use of initially non-zeroed
/// mutable statics.
///
/// See also [`bss_init`].
///
/// # Safety
///
/// * Calling this function after mutating statics effectively reverts them to
///   the initial state.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub unsafe fn data_init(start: &mut usize, end: &usize, data: &usize) {
    let start = start as *mut _;
    let end = end as *const _;
    let data = data as *const _;
    let count = end as usize - start as usize;
    ptr::copy_nonoverlapping(data, start, count >> 2);
}
