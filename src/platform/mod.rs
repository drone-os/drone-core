//! CPU management.

#![cfg_attr(feature = "std", allow(dead_code, unreachable_code))]

mod interrputs;

pub use self::interrputs::Interrupts;

extern "C" {
    fn drone_reset() -> !;
    fn drone_save_and_disable_interrupts() -> u32;
    fn drone_restore_interrupts(status: u32);
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
