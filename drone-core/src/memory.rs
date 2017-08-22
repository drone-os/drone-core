//! Basic functions for dealing with memory.


use core::ptr;


/// Initializes the `.bss` section.
///
/// # Safety
///
/// Must be called exactly once and as early as possible.
pub unsafe fn bss_init() {
  extern "C" {
    static mut BSS_START: usize;
    static BSS_END: usize;
  }
  let bss_start = &mut BSS_START as *mut usize;
  let bss_end = &BSS_END as *const usize;
  let count = (bss_end as usize - bss_start as usize) >> 2;
  ptr::write_bytes(bss_start, 0, count);
}


/// Initializes the `.data` section.
///
/// # Safety
///
/// Must be called exactly once and as early as possible.
pub unsafe fn data_init() {
  extern "C" {
    static mut DATA_START: usize;
    static DATA_END: usize;
    static DATA_CONST: usize;
  }
  let data_start = &mut DATA_START as *mut usize;
  let data_end = &DATA_END as *const usize;
  let data_const = &DATA_CONST as *const usize;
  let count = (data_end as usize - data_start as usize) >> 2;
  ptr::copy_nonoverlapping(data_const, data_start, count);
}
