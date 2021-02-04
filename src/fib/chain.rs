use crate::{
    fib::RootFiber,
    sync::linked_list::{DrainFilter, LinkedList},
};
use core::pin::Pin;

/// A lock-free stack of fibers.
pub struct Chain {
    stack: LinkedList<Node>,
}

type Node = Pin<Box<dyn RootFiber>>;

impl Chain {
    /// Creates an empty fiber chain.
    #[inline]
    pub const fn new() -> Self {
        Self { stack: LinkedList::new() }
    }

    /// Adds a fiber first in the chain.
    #[inline]
    pub fn add<F: RootFiber>(&self, fib: F) {
        self.stack.push(Box::pin(fib));
    }

    /// Returns `true` if the [`Chain`] is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Returns an iterator that advances each fiber in the chain, returning
    /// completed ones.
    ///
    /// # Examples
    ///
    /// The returned iterator can be simply dropped, it's destructor will drop
    /// all completed fibers:
    ///
    /// ```
    /// use drone_core::fib::Chain;
    ///
    /// let chain = Chain::new();
    /// unsafe {
    ///     drop(chain.drain()); // run the iterator and drop completed fibers
    /// }
    /// ```
    ///
    /// Check if there is at least one active fiber:
    ///
    /// ```
    /// use drone_core::fib::Chain;
    ///
    /// let chain = Chain::new();
    /// let drain = unsafe { chain.drain() };
    /// if drain.is_end() {
    ///     println!("No active fibers to react to this interrupt");
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// This method must not be called again when the previous iterator is still
    /// alive.
    #[inline]
    pub unsafe fn drain(&self) -> DrainFilter<'_, Node, impl FnMut(&mut Node) -> bool> {
        // This is the only place where nodes are getting removed, and this
        // function is not running concurrently because of the safety invariant
        // of this function.
        unsafe { self.stack.drain_filter_unchecked(|fib| fib.as_mut().advance()) }
    }
}
