use core::cell::Cell;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;
use core::{mem, ptr};
use futures::task;
use thr::prelude::*;
use thr::{current, ThreadLocal};

static CURRENT: AtomicUsize = AtomicUsize::new(0);

type StaticContext = *mut task::Context<'static>;

/// A thread-local storage of the task pointer.
pub struct TaskCell(Cell<StaticContext>);

struct Reset<'a>(StaticContext, &'a Cell<StaticContext>);

impl TaskCell {
  /// Creates a new `TaskCell`.
  pub const fn new() -> Self {
    TaskCell(Cell::new(ptr::null_mut()))
  }

  #[cfg_attr(feature = "clippy", allow(useless_transmute))]
  pub(crate) fn __set_cx<F, T>(&self, cx: &mut task::Context, f: F) -> T
  where
    F: FnOnce() -> T,
  {
    let prev_cx = self.0.replace(unsafe {
      mem::transmute::<*mut task::Context<'_>, *mut task::Context<'static>>(cx)
    });
    let _r = Reset(prev_cx, &self.0);
    f()
  }

  #[doc(hidden)]
  pub fn __in_cx<F, T>(&self, f: F) -> T
  where
    F: FnOnce(&mut task::Context) -> T,
  {
    let cx = self.0.replace(ptr::null_mut());
    if cx.is_null() {
      panic!("not an async context")
    } else {
      let _r = Reset(cx, &self.0);
      f(unsafe { &mut *cx })
    }
  }
}

impl<'a> Drop for Reset<'a> {
  fn drop(&mut self) {
    self.1.set(self.0);
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
