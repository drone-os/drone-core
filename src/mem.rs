//! Basic functions for dealing with memory.

use core::cell::UnsafeCell;
use core::ptr;

extern "C" {
    static BSS_BASE: UnsafeCell<usize>;
    static BSS_END: UnsafeCell<usize>;
    static DATA_LOAD: UnsafeCell<usize>;
    static DATA_BASE: UnsafeCell<usize>;
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
    unsafe { zeroed_section_init(&BSS_BASE, &BSS_END) };
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
    unsafe { data_section_init(&DATA_LOAD, &DATA_BASE, &DATA_END) };
}

/// Initializes a zeroed section in RAM memory.
///
/// See also [`bss_init`].
///
/// # Examples
///
/// ```no_run
/// use core::cell::UnsafeCell;
/// use drone_core::mem;
///
/// extern "C" {
///     static BSS_BASE: UnsafeCell<usize>;
///     static BSS_END: UnsafeCell<usize>;
/// }
///
/// unsafe {
///     mem::zeroed_section_init(&BSS_BASE, &BSS_END);
/// }
/// ```
///
/// # Safety
///
/// This function is very unsafe, because it directly overwrites the memory.
pub unsafe fn zeroed_section_init(base: &UnsafeCell<usize>, end: &UnsafeCell<usize>) {
    unsafe {
        let length = end.get() as usize - base.get() as usize;
        ptr::write_bytes(base.get(), 0, length >> 2);
    }
}

/// Initializes a data section in RAM memory.
///
/// See also [`data_init`].
///
/// # Examples
///
/// ```no_run
/// use core::cell::UnsafeCell;
/// use drone_core::mem;
///
/// extern "C" {
///     static DATA_LOAD: UnsafeCell<usize>;
///     static DATA_BASE: UnsafeCell<usize>;
///     static DATA_END: UnsafeCell<usize>;
/// }
///
/// unsafe {
///     mem::data_section_init(&DATA_LOAD, &DATA_BASE, &DATA_END);
/// }
/// ```
///
/// # Safety
///
/// This function is very unsafe, because it directly overwrites the memory.
pub unsafe fn data_section_init(
    load: &UnsafeCell<usize>,
    base: &UnsafeCell<usize>,
    end: &UnsafeCell<usize>,
) {
    unsafe {
        let length = end.get() as usize - base.get() as usize;
        ptr::copy_nonoverlapping(load.get(), base.get(), length >> 2);
    }
}
