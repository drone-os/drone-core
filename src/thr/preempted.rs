use crate::thr::Thread;
use core::cell::Cell;

static mut CURRENT: usize = 0;

/// Thread-local previous thread index cell.
pub struct PreemptedCell(Cell<usize>);

impl PreemptedCell {
    /// Creates a new `PreemptedCell`.
    pub const fn new() -> Self {
        Self(Cell::new(0))
    }
}

/// Returns a reference to the thread-local storage of the current thread.
///
/// The contents of this object can be customized with `thr!` macro. See [`the
/// module-level documentation`][`crate::thr`] for details.
#[inline]
pub fn local<T: Thread>() -> &'static T::Local {
    unsafe { (*T::first().add(CURRENT)).local() }
}

pub unsafe fn preempt(preempted: &PreemptedCell, thr_num: usize, f: impl FnOnce()) {
    preempted.0.set(CURRENT);
    CURRENT = thr_num;
    f();
    CURRENT = preempted.0.get();
}
