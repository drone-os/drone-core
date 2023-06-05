#![cfg_attr(feature = "host", allow(unused_imports, unused_variables))]

use super::{drone_restore_interrupts, drone_save_and_disable_interrupts};

/// Critical section.
///
/// A critical section is a block of code surrounded by interrupts disable and
/// interrupts re-enable instructions. This type disables interrupts on creation
/// and re-enables them (if were enabled) on drop. Critical sections are allowed
/// to be nested.
///
/// Critical sections are useful to implement atomic operations in exchange of
/// delaying execution of higher priority threads (interrupts). Therefore the
/// code in a critical section should be as minimal as possible.
///
/// # Priority inversion hazard
///
/// On devices such as RP2040, which don't have a built-in flash memory, but use
/// XIP (eXecute In Place) mechanism instead, there could be a state, when a
/// lower priority thread executes a code from an external flash via XIP. The
/// code enters a critical section and a XIP cache miss occurs before interrupts
/// are re-enabled. Other higher priority threads can't be executed during XIP
/// cache load, this is called priority inversion.
///
/// To mitigate this issue, there is a cargo feature for `drone-core` called
/// `xip`. When enabled, [`Interrupts::paused`] is marked `#[inline(never)]` and
/// `#[link_section = ".time_critical"]`. All code in `.time_critical` link
/// section is first copied into RAM at application startup. Care must be taken
/// to not access anything from XIP memory region inside the critical section.
/// Code executing purely in RAM is not subject to this kind of priority
/// inversion.
///
/// If using [`Interrupts::pause`], actions from the above paragraph should be
/// taken manually.
pub struct Interrupts {
    save: u32,
}

impl Interrupts {
    /// Creates a new critical section handle.
    ///
    /// This function disables all interrupts for the current CPU. Interrupts
    /// are re-enabled when this instance is dropped (unless it's nested into
    /// another critical section).
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
    #[inline]
    pub fn pause() -> Self {
        Self {
            #[cfg(feature = "host")]
            save: 0,
            #[cfg(not(feature = "host"))]
            save: unsafe { drone_save_and_disable_interrupts() },
        }
    }

    /// Runs a closure inside a critical section.
    ///
    /// All interrupts for the current CPU are disabled before executing the
    /// closure and re-enabled after the closure has executed (unless it's
    /// nested into another critical section).
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
    #[cfg_attr(not(feature = "xip"), inline)]
    #[cfg_attr(feature = "xip", inline(never))]
    #[cfg_attr(feature = "xip", link_section = ".time_critical")]
    pub fn paused<R, F: FnOnce() -> R>(f: F) -> R {
        let _paused = Self::pause();
        f()
    }
}

impl Drop for Interrupts {
    fn drop(&mut self) {
        let Self { save } = *self;
        #[cfg(not(feature = "host"))]
        unsafe {
            drone_restore_interrupts(save);
        }
    }
}
