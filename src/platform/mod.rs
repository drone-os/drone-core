//! CPU management.

#![cfg_attr(feature = "host", allow(dead_code, unreachable_code, unused_variables))]

mod interrputs;

pub use self::interrputs::Interrupts;
use core::cell::UnsafeCell;
use drone_stream::Runtime;

extern "C" {
    fn drone_reset() -> !;
    fn drone_save_and_disable_interrupts() -> u32;
    fn drone_restore_interrupts(status: u32);
    fn drone_data_mem_init(load: *const usize, base: *mut usize, end: *const usize);
    fn drone_zeroed_mem_init(base: *mut usize, end: *const usize);
    fn drone_stream_runtime() -> *mut Runtime;
}

/// Runs a predicate in a tight loop. Stops when the predicate returns `false`.
///
/// This is an equivalent to `while f() {}`. Using this ubiquitously makes it
/// much easier to find tight loops.
///
/// See also [`spin_until`](crate::spin_until).
///
/// # Examples
///
/// ```
/// use drone_core::spin_while;
/// let mut i = 0;
/// let mut poll = || {
///     i += 1;
///     i
/// };
/// spin_while!(poll() < 10);
/// ```
#[macro_export]
macro_rules! spin_while {
    ($pred:expr) => {
        while $pred {}
    };
}

/// Runs a predicate in a tight loop. Stops when the predicate returns `true`.
///
/// This is an equivalent to `while !f() {}`. Using this ubiquitously makes it
/// much easier to find tight loops.
///
/// See also [`spin_while`](crate::spin_while).
///
/// # Examples
///
/// ```
/// use drone_core::spin_until;
/// let mut i = 0;
/// let mut poll = || {
///     i += 1;
///     i
/// };
/// spin_until!(poll() == 10);
/// ```
#[macro_export]
macro_rules! spin_until {
    ($pred:expr) => {
        $crate::spin_while!(!$pred)
    };
}

/// Requests system reset.
///
/// This function never returns.
#[inline]
pub fn reset() -> ! {
    #[cfg(feature = "host")]
    return unimplemented!();
    #[cfg(not(feature = "host"))]
    unsafe {
        drone_reset()
    }
}

/// Fills a memory region with zeros without using compiler built-ins.
///
/// See also [`data_mem_init`].
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
///     platform::zeroed_mem_init(&BSS_BASE, &BSS_END);
/// }
/// ```
///
/// # Safety
///
/// This function is very unsafe, because it directly overwrites the memory.
#[inline]
pub unsafe fn zeroed_mem_init(base: &UnsafeCell<usize>, end: &UnsafeCell<usize>) {
    // Need to use assembly code, because pure Rust code can be optimized to use the
    // compiler builtin `memcpy`, which may be not available yet.
    #[cfg(feature = "host")]
    return unimplemented!();
    #[cfg(not(feature = "host"))]
    unsafe {
        drone_zeroed_mem_init(base.get(), end.get());
    }
}

/// Copies bytes from one memory region to another without using compiler
/// built-ins.
///
/// See also [`zeroed_mem_init`].
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
///     platform::data_mem_init(&DATA_LOAD, &DATA_BASE, &DATA_END);
/// }
/// ```
///
/// # Safety
///
/// This function is very unsafe, because it directly overwrites the memory.
#[inline]
pub unsafe fn data_mem_init(
    load: &UnsafeCell<usize>,
    base: &UnsafeCell<usize>,
    end: &UnsafeCell<usize>,
) {
    // Need to use assembly code, because pure Rust code can be optimized to use the
    // compiler builtin `memset`, which may be not available yet.
    #[cfg(feature = "host")]
    return unimplemented!();
    #[cfg(not(feature = "host"))]
    unsafe {
        drone_data_mem_init(load.get(), base.get(), end.get());
    }
}

/// Returns a mutable reference to the Drone Stream runtime.
#[inline]
pub fn stream_rt() -> *mut Runtime {
    #[cfg(feature = "host")]
    return unimplemented!();
    #[cfg(not(feature = "host"))]
    unsafe {
        drone_stream_runtime()
    }
}
