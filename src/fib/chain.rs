use core::iter::FusedIterator;
use core::pin::Pin;

use crate::fib::RootFiber;
use crate::sync::linked_list::{DrainFilterRaw, LinkedList, Node as ListNode};

/// A lock-free list of fibers.
pub struct Chain {
    list: LinkedList<Node<()>>,
}

#[repr(C)]
pub struct Node<F> {
    advance: unsafe fn(*mut ListNode<Node<()>>) -> bool,
    deallocate: unsafe fn(*mut ListNode<Node<()>>),
    fib: F,
}

/// An iterator produced by [`Chain::drain`].
pub struct Drain<'a, F>
where
    F: FnMut(*mut ListNode<Node<()>>) -> bool,
{
    inner: DrainFilterRaw<'a, Node<()>, F>,
}

impl Chain {
    /// Creates an empty fiber chain.
    #[inline]
    pub const fn new() -> Self {
        Self { list: LinkedList::new() }
    }

    /// Adds a fiber first in the chain.
    #[inline]
    pub fn add<F: RootFiber>(&self, fib: F) {
        unsafe { self.list.push_raw(Node::allocate(fib)) };
    }

    /// Returns `true` if the chain is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
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
    ///     chain.drain().for_each(drop); // run the iterator and drop completed fibers
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
    /// drain.for_each(drop);
    /// ```
    ///
    /// # Safety
    ///
    /// This method must not be called again when the previous iterator is still
    /// alive.
    #[inline]
    pub unsafe fn drain(&self) -> Drain<'_, impl FnMut(*mut ListNode<Node<()>>) -> bool> {
        // This is the only place where nodes are getting removed. This cannot
        // run concurrently because of the safety invariant of this function.
        unsafe { Drain { inner: self.list.drain_filter_raw(Node::filter) } }
    }
}

impl Drop for Chain {
    #[inline]
    fn drop(&mut self) {
        unsafe { self.list.drain_filter_raw(|_| true).for_each(Node::delete) };
    }
}

impl Node<()> {
    fn filter(node: *mut ListNode<Self>) -> bool {
        unsafe { ((*node).advance)(node) }
    }

    fn delete(node: *mut ListNode<Self>) {
        unsafe { ((*node).deallocate)(node) }
    }
}

impl<F: RootFiber> Node<F> {
    fn allocate(fib: F) -> *mut ListNode<Node<()>> {
        let node = Node { advance: Self::advance, deallocate: Self::deallocate, fib };
        unsafe { Self::upcast(Box::into_raw(Box::new(ListNode::from(node)))) }
    }

    unsafe fn advance(node: *mut ListNode<Node<()>>) -> bool {
        unsafe { Pin::new_unchecked(&mut (*Self::downcast(node)).fib).advance() }
    }

    unsafe fn deallocate(node: *mut ListNode<Node<()>>) {
        unsafe { Box::from_raw(Self::downcast(node)) };
    }

    unsafe fn upcast(node: *mut ListNode<Self>) -> *mut ListNode<Node<()>> {
        node.cast()
    }

    unsafe fn downcast(node: *mut ListNode<Node<()>>) -> *mut ListNode<Self> {
        node.cast()
    }
}

impl<F> Drain<'_, F>
where
    F: FnMut(*mut ListNode<Node<()>>) -> bool,
{
    /// Returns `true` if there are no fibers left in the chain.
    #[inline]
    pub fn is_end(&self) -> bool {
        self.inner.is_end()
    }
}

impl<F> Iterator for Drain<'_, F>
where
    F: FnMut(*mut ListNode<Node<()>>) -> bool,
{
    type Item = ();

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Node::delete)
    }
}

impl<F> FusedIterator for Drain<'_, F> where F: FnMut(*mut ListNode<Node<()>>) -> bool {}
