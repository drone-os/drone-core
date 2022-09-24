#![cfg_attr(feature = "std", allow(unused_imports, unused_variables))]

use super::{drone_restore_interrupts, drone_save_and_disable_interrupts};

/// Critical section.
///
/// This is a ZST (zero-sized type) with the following semantics: while an
/// instance of it exists, the current thread can't be pre-empted by other
/// (even higher priority) threads.
///
/// When this type is created, interrupts are disabled. Interrupts are enabled
/// back automatically when the instance is dropped.
pub struct Interrupts {
    save: u32,
}

impl Interrupts {
    /// Creates a new critical section handle.
    ///
    /// This function disables all interrupts for the current CPU. Interrupts
    /// are re-enabled when the instance is dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::platform::Interrupts;
    ///
    /// let mut x = 0;
    /// {
    ///     // Making this block of code un-interruptable by creating a new value of the
    ///     // `Interrupts` type. The value is dropped at the end of this block.
    ///     let _critical = Interrupts::pause();
    ///     x += 1;
    /// }
    /// dbg!(x);
    /// ```
    pub fn pause() -> Self {
        Self {
            #[cfg(feature = "std")]
            save: 0,
            #[cfg(not(feature = "std"))]
            save: unsafe { drone_save_and_disable_interrupts() },
        }
    }

    /// Runs a closure inside a critical section.
    ///
    /// All interrupts for the current CPU are disabled before executing the
    /// closure and re-enabled after the closure has executed.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::platform::Interrupts;
    ///
    /// let mut x = 0;
    /// Interrupts::paused(|| {
    ///     // This block of code is un-interruptable.
    ///     x += 1;
    /// });
    /// dbg!(x);
    /// ```
    pub fn paused<R, F: FnOnce() -> R>(f: F) -> R {
        let _paused = Self::pause();
        f()
    }
}

impl Drop for Interrupts {
    fn drop(&mut self) {
        let Self { save } = *self;
        #[cfg(not(feature = "std"))]
        unsafe {
            drone_restore_interrupts(save);
        }
    }
}
