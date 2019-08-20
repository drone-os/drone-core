use core::{cell::Cell, mem::transmute, ptr::NonNull, task::Context};

/// Thread-local task context cell.
pub struct TaskCell(Cell<TaskContext>);

type TaskContext = Option<NonNull<Context<'static>>>;

struct ResetContext<'a>(TaskContext, &'a Cell<TaskContext>);

impl TaskCell {
    /// Creates a new task context cell.
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
