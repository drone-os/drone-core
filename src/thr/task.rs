use core::cell::Cell;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;
use core::{mem, ptr};
use futures::task;
use thr::prelude::*;
use thr::{current, ThreadLocal};

static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// A thread-local storage of the task pointer.
pub struct TaskCell(Cell<*mut task::Context<'static>>);

impl TaskCell {
  /// Creates a new `TaskCell`.
  pub const fn new() -> Self {
    TaskCell(Cell::new(ptr::null_mut()))
  }

  pub(crate) fn __set_cx<F, T>(&self, cx: &mut task::Context, f: F) -> T
  where
    F: FnOnce() -> T,
  {
    let prev_cx = self.0.replace(unsafe { mem::transmute(cx) });
    let result = f();
    self.0.set(prev_cx);
    result
  }

  #[doc(hidden)]
  pub fn __in_cx<F, T>(&self, f: F) -> T
  where
    F: FnOnce(&mut task::Context) -> T,
  {
    let cx = self.0.replace(ptr::null_mut());
    let result = if cx == ptr::null_mut() {
      panic!("not an async context")
    } else {
      f(unsafe { &mut *cx })
    };
    self.0.set(cx);
    result
  }
}

/// Initializes the `futures` task system.
///
/// # Safety
///
/// Must be called before using `futures`.
#[inline(always)]
pub unsafe fn init<T: Thread>() {
  CURRENT.store(current_task_fn::<T> as usize, Relaxed);
}

#[doc(hidden)]
pub fn __current_task() -> &'static TaskCell {
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
