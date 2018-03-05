use core::cell::Cell;
use core::ptr;
use futures::task;
use thr::{current, ThreadLocal};
use thr::prelude::*;

/// A thread-local storage of the task pointer.
pub struct TaskCell(Cell<*mut u8>);

impl TaskCell {
  /// Creates a new `TaskCell`.
  pub const fn new() -> Self {
    TaskCell(Cell::new(ptr::null_mut()))
  }
}

/// Initializes the `futures` task system.
///
/// # Safety
///
/// Must be called before using `futures`.
#[inline(always)]
pub unsafe fn init<T: Thread>() -> bool {
  task::init(get_task::<T>, set_task::<T>)
}

fn get_task<T: Thread>() -> *mut u8 {
  current::<T>().task().0.get()
}

fn set_task<T: Thread>(task: *mut u8) {
  current::<T>().task().0.set(task);
}
