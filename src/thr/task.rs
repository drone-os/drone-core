use crate::thr::{current, prelude::*, ThreadLocal};
use core::{
    cell::Cell,
    mem::transmute,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering::*},
    task::Context,
};

static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// A thread-local storage of the task pointer.
pub struct TaskCell(Cell<TaskContext>);

type TaskContext = Option<NonNull<Context<'static>>>;

struct ResetContext<'a>(TaskContext, &'a Cell<TaskContext>);

impl TaskCell {
    /// Creates a new `TaskCell`.
    pub const fn new() -> Self {
        Self(Cell::new(None))
    }

    pub(crate) fn set_context<F, R>(&self, cx: &mut Context<'_>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let cx = unsafe { transmute::<&mut Context<'_>, &mut Context<'static>>(cx) };
        let prev_cx = self.0.replace(Some(NonNull::from(cx)));
        let _reset = ResetContext(prev_cx, &self.0);
        f()
    }

    pub(crate) fn get_context<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Context<'_>) -> R,
    {
        let cx = self.0.replace(None);
        let _reset = ResetContext(cx, &self.0);
        f(unsafe { cx.expect("not an async context").as_mut() })
    }
}

impl<'a> Drop for ResetContext<'a> {
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
        unsafe { transmute::<usize, fn() -> &'static TaskCell>(ptr)() }
    }
}

fn current_task_fn<T: Thread>() -> &'static TaskCell {
    current::<T>().task()
}
