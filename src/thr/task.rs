use crate::thr::{current, prelude::*, ThreadLocal};
use core::{
  cell::Cell,
  mem, ptr,
  sync::atomic::{AtomicUsize, Ordering::*},
  task::Waker,
};

static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// A thread-local storage of the task pointer.
pub struct TaskCell(Cell<TaskWaker>);

type TaskWaker = *const Waker;

struct ResetWaker<'a>(TaskWaker, &'a Cell<TaskWaker>);

impl TaskCell {
  /// Creates a new `TaskCell`.
  pub const fn new() -> Self {
    Self(Cell::new(ptr::null_mut()))
  }

  pub(crate) fn set_waker<F, R>(&self, waker: &Waker, f: F) -> R
  where
    F: FnOnce() -> R,
  {
    let prev_lw = self.0.replace(waker);
    let _reset_lw = ResetWaker(prev_lw, &self.0);
    f()
  }

  pub(crate) fn get_waker<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&Waker) -> R,
  {
    let waker = self.0.replace(ptr::null_mut());
    if waker.is_null() {
      panic!("not an async context")
    } else {
      let _reset_lw = ResetWaker(waker, &self.0);
      f(unsafe { &*waker })
    }
  }
}

impl<'a> Drop for ResetWaker<'a> {
  fn drop(&mut self) {
    self.1.set(self.0);
  }
}

/// Initializes the `futures` task system.
///
/// # Safety
///
/// Must be called before using `futures`.
pub unsafe fn init<T: Thread>() {
  CURRENT.store(current_task_fn::<T> as usize, Relaxed);
}

#[doc(hidden)]
pub fn current_task() -> &'static TaskCell {
  let ptr = CURRENT.load(Relaxed);
  if ptr == 0 {
    panic!("not initialized");
  } else {
    unsafe { mem::transmute::<usize, fn() -> &'static TaskCell>(ptr)() }
  }
}

fn current_task_fn<T: Thread>() -> &'static TaskCell {
  current::<T>().task()
}
