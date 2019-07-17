//! Basic functions for dealing with memory.

use core::ptr;

/// Initializes the `.bss` section.
///
/// # Safety
///
/// * Must be called no more than once.
/// * Must be called before accessing `static`s.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub unsafe fn bss_init(start: &mut usize, end: &usize) {
    let start = start as *mut _;
    let end = end as *const _;
    let count = end as usize - start as usize;
    ptr::write_bytes(start, 0, count >> 2);
}

/// Initializes the `.data` section.
///
/// # Safety
///
/// * Must be called no more than once.
/// * Must be called before accessing `static`s.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub unsafe fn data_init(start: &mut usize, end: &usize, data: &usize) {
    let start = start as *mut _;
    let end = end as *const _;
    let data = data as *const _;
    let count = end as usize - start as usize;
    ptr::copy_nonoverlapping(data, start, count >> 2);
}
