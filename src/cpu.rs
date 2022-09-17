//! CPU management.

#![cfg_attr(feature = "std", allow(dead_code, unreachable_code))]

extern "C" {
    fn drone_self_reset() -> !;
    fn drone_int_enable();
    fn drone_int_disable();
}

/// Critical section.
///
/// This is a ZST (zero-sized type) with the following semantics: while an
/// instance of it exists, the current thread can't be pre-empted by other
/// (even higher priority) threads.
///
/// When this type is created, interrupts are disabled. Interrupts are enabled
/// back automatically when the instance is dropped.
pub struct Critical(());

impl Critical {
    /// Creates a new critical section handle.
    ///
    /// This function disables all interrupts for the current CPU. Interrupts
    /// are re-enabled when the instance is dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::cpu::Critical;
    ///
    /// let mut x = 0;
    /// {
    ///     // Making this block of code un-interruptable by creating a new
    ///     // value of the `Critical` type. The value is dropped at the end of
    ///     // this block.
    ///     let _critical = Critical::enter();
    ///     x += 1;
    /// }
    /// dbg!(x);
    /// ```
    pub fn enter() -> Self {
        #[cfg(not(feature = "std"))]
        unsafe {
            drone_int_disable();
        }
        Self(())
    }

    /// Runs a closure inside a critical section.
    ///
    /// All interrupts for the current CPU are disabled before executing the
    /// closure and re-enabled after the closure has executed.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::cpu::Critical;
    ///
    /// let mut x = 0;
    /// Critical::section(|| {
    ///     // This block of code is un-interruptable.
    ///     x += 1;
    /// });
    /// dbg!(x);
    /// ```
    pub fn section<R, F: FnOnce() -> R>(f: F) -> R {
        let _critical = Self::enter();
        f()
    }
}

impl Drop for Critical {
    fn drop(&mut self) {
        #[cfg(not(feature = "std"))]
        unsafe {
            drone_int_enable();
        }
    }
}

/// Requests system reset.
///
/// This function never returns.
#[inline]
pub fn self_reset() -> ! {
    #[cfg(feature = "std")]
    return unimplemented!();
    #[cfg(not(feature = "std"))]
    unsafe {
        drone_self_reset()
    }
}
