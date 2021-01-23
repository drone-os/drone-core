use crate::fib::FiberRoot;
use core::{
    pin::Pin,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

/// A lock-free stack of fibers.
pub struct Chain {
    head: AtomicPtr<Node>,
}

struct Node {
    fib: Pin<Box<dyn FiberRoot>>,
    next: *mut Node,
}

impl Chain {
    /// Creates an empty fiber chain.
    pub const fn new() -> Self {
        Self { head: AtomicPtr::new(ptr::null_mut()) }
    }

    /// Adds the fiber `fib` first to the chain.
    pub fn add<F: FiberRoot>(&self, fib: F) {
        self.push(Node::new(fib));
    }

    /// Returns `true` if the chain contains no fibers.
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }

    /// Advances fibers, removing completed ones. Returned `bool` indicates
    /// whether any fiber was executed.
    ///
    /// # Safety
    ///
    /// This method is not reentrant.
    #[inline(never)]
    pub unsafe fn drain(&self) -> bool {
        let mut advanced = false;
        let mut prev = ptr::null_mut();
        let mut curr = self.head.load(Ordering::Acquire);
        while !curr.is_null() {
            unsafe {
                let next = (*curr).next;
                if (*curr).fib.as_mut().advance() {
                    advanced = true;
                    prev = curr;
                } else {
                    if prev.is_null() {
                        if self
                            .head
                            .compare_exchange(curr, next, Ordering::Relaxed, Ordering::Relaxed)
                            .is_ok()
                        {
                            prev = ptr::null_mut();
                        } else {
                            loop {
                                prev = (*prev).next;
                                if prev == curr {
                                    (*prev).next = next;
                                    break;
                                }
                            }
                        }
                    } else {
                        (*prev).next = next;
                    }
                    drop(Box::from_raw(curr));
                }
                curr = next;
            }
        }
        advanced
    }

    fn push(&self, node: Node) {
        let node = Box::into_raw(Box::new(node));
        loop {
            let head = self.head.load(Ordering::Relaxed);
            unsafe { (*node).next = head };
            if self
                .head
                .compare_exchange_weak(head, node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }
}

impl Node {
    fn new<F: FiberRoot>(fib: F) -> Self {
        Self { fib: Box::pin(fib), next: ptr::null_mut() }
    }
}
