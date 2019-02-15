use crate::thr::Thread;
use core::cell::Cell;

static mut CURRENT: usize = 0;

/// A thread-local storage of the preempted thread index.
pub struct PreemptedCell(Cell<usize>);

impl PreemptedCell {
  /// Creates a new `PreemptedCell`.
  pub const fn new() -> Self {
    Self(Cell::new(0))
  }
}

/// Returns a static reference to the current thread.
#[inline]
pub fn current<T: Thread>() -> &'static T::Local {
  unsafe { (*T::first().add(CURRENT)).get_local() }
}

/// Sets a context for [`current`](current) within `f`.
///
/// # Safety
///
/// Must be called once at the beginning of the thread handler.
pub unsafe fn with_preempted(
  preempted: &PreemptedCell,
  thr_num: usize,
  f: impl FnOnce(),
) {
  preempted.0.set(CURRENT);
  CURRENT = thr_num;
  f();
  CURRENT = preempted.0.get();
}
