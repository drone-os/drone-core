//! CPU management.

#![cfg_attr(feature = "std", allow(dead_code, unreachable_code, unused_variables))]

mod interrputs;

pub use self::interrputs::Interrupts;
use core::cell::UnsafeCell;

extern "C" {
    fn drone_reset() -> !;
    fn drone_save_and_disable_interrupts() -> u32;
    fn drone_restore_interrupts(status: u32);
    fn drone_data_section_init(load: *const usize, base: *mut usize, end: *const usize);
    fn drone_zeroed_section_init(base: *mut usize, end: *const usize);
}

/// Requests system reset.
///
/// This function never returns.
#[inline]
pub fn reset() -> ! {
    #[cfg(feature = "std")]
    return unimplemented!();
    #[cfg(not(feature = "std"))]
    unsafe {
        drone_reset()
    }
}

/// Initializes a zeroed section in RAM memory.
///
/// See also [`data_section_init`].
///
/// # Examples
///
/// ```no_run
/// use core::cell::UnsafeCell;
/// use drone_core::platform;
///
/// extern "C" {
///     static BSS_BASE: UnsafeCell<usize>;
///     static BSS_END: UnsafeCell<usize>;
/// }
///
/// unsafe {
///     platform::zeroed_section_init(&BSS_BASE, &BSS_END);
/// }
/// ```
///
/// # Safety
///
/// This function is very unsafe, because it directly overwrites the memory.
#[inline]
pub unsafe fn zeroed_section_init(base: &UnsafeCell<usize>, end: &UnsafeCell<usize>) {
    // Need to use assembly code, because pure Rust code can be optimized to use the
    // compiler builtin `memcpy`, which may be not available yet.
    #[cfg(feature = "std")]
    return unimplemented!();
    #[cfg(not(feature = "std"))]
    unsafe {
        drone_zeroed_section_init(base.get(), end.get());
    }
}

/// Initializes a data section in RAM memory.
///
/// See also [`zeroed_section_init`].
///
/// # Examples
///
/// ```no_run
/// use core::cell::UnsafeCell;
/// use drone_core::platform;
///
/// extern "C" {
///     static DATA_LOAD: UnsafeCell<usize>;
///     static DATA_BASE: UnsafeCell<usize>;
///     static DATA_END: UnsafeCell<usize>;
/// }
///
/// unsafe {
///     platform::data_section_init(&DATA_LOAD, &DATA_BASE, &DATA_END);
/// }
/// ```
///
/// # Safety
///
/// This function is very unsafe, because it directly overwrites the memory.
#[inline]
pub unsafe fn data_section_init(
    load: &UnsafeCell<usize>,
    base: &UnsafeCell<usize>,
    end: &UnsafeCell<usize>,
) {
    // Need to use assembly code, because pure Rust code can be optimized to use the
    // compiler builtin `memset`, which may be not available yet.
    #[cfg(feature = "std")]
    return unimplemented!();
    #[cfg(not(feature = "std"))]
    unsafe {
        drone_data_section_init(load.get(), base.get(), end.get());
    }
}
