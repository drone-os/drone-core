use crate::fib::FiberRoot;
use core::{
  pin::Pin,
  ptr,
  sync::atomic::{AtomicPtr, Ordering::*},
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
  /// Creates an empty `Chain`.
  pub const fn new() -> Self {
    Self {
      head: AtomicPtr::new(ptr::null_mut()),
    }
  }

  /// Adds a fiber first in the chain.
  pub fn add<F: FiberRoot>(&self, fib: F) {
    self.push(Node::new(fib));
  }

  /// Returns `true` if the chain contains no fibers.
  pub fn is_empty(&self) -> bool {
    self.head.load(Acquire).is_null()
  }

  /// Advances all fibers, removing completed ones.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  #[inline(never)]
  pub unsafe fn drain(&self) {
    let mut prev = ptr::null_mut();
    let mut curr = self.head.load(Acquire);
    while !curr.is_null() {
      let next = (*curr).next;
      if (*curr).fib.as_mut().advance() {
        prev = curr;
      } else {
        if prev.is_null() {
          prev = self.head.compare_and_swap(curr, next, Relaxed);
          if prev == curr {
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

  fn push(&self, node: Node) {
    let node = Box::into_raw(Box::new(node));
    loop {
      let head = self.head.load(Relaxed);
      unsafe { (*node).next = head };
      if self.head.compare_and_swap(head, node, Release) == head {
        break;
      }
    }
  }
}

impl Node {
  fn new<F: FiberRoot>(fib: F) -> Self {
    Self {
      fib: Box::pin(fib),
      next: ptr::null_mut(),
    }
  }
}
