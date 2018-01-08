use super::current;
use core::cell::UnsafeCell;
use core::ptr;
use futures::task;
use thread::prelude::*;

/// A thread-local storage of the task pointer.
pub struct TaskCell(UnsafeCell<*mut u8>);

/// Initialize the `futures` task system.
///
/// # Safety
///
/// Must be called before using `futures`.
#[inline(always)]
pub unsafe fn init<T: Thread>() -> bool {
  task::init(get_task::<T>, set_task::<T>)
}

fn get_task<T: Thread>() -> *mut u8 {
  unsafe { current::<T>().task().get() }
}

fn set_task<T: Thread>(task: *mut u8) {
  unsafe { current::<T>().task().set(task) };
}

impl TaskCell {
  /// Creates a new `TaskCell`.
  #[inline(always)]
  pub const fn new() -> Self {
    TaskCell(UnsafeCell::new(ptr::null_mut()))
  }

  #[inline(always)]
  unsafe fn get(&self) -> *mut u8 {
    *self.0.get()
  }

  #[inline(always)]
  unsafe fn set(&self, task: *mut u8) {
    *self.0.get() = task;
  }
}

unsafe impl Sync for TaskCell {}
