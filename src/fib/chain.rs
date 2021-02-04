use crate::{fib::FiberRoot, sync::LinkedList};
use core::pin::Pin;

/// A lock-free stack of fibers.
pub struct Chain {
    stack: LinkedList<Pin<Box<dyn FiberRoot>>>,
}

impl Chain {
    /// Creates an empty fiber chain.
    pub const fn new() -> Self {
        Self { stack: LinkedList::new() }
    }

    /// Adds a fiber first in the chain.
    pub fn add<F: FiberRoot>(&self, fib: F) {
        self.stack.push(Box::pin(fib));
    }

    /// Returns `true` if the [`Chain`] is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Advances fibers, removing completed ones. Returned `bool` indicates
    /// whether at least one fiber has been executed.
    ///
    /// # Safety
    ///
    /// This method is not re-entrant.
    #[inline(never)]
    pub unsafe fn drain(&self) -> bool {
        // This is the only place where nodes are getting removed, and this
        // function is not running concurrently because of the safety invariant
        // of this function.
        let drain_filter =
            unsafe { self.stack.drain_filter_unchecked(|fib| fib.as_mut().advance()) };
        !drain_filter.is_end()
    }
}
