//! Stackless semicoroutines.

mod future;
mod stream_ring;
mod stream_unit;

pub use self::future::FiberFuture;
pub use self::stream_ring::FiberStreamRing;
pub use self::stream_unit::FiberStreamUnit;

use core::ptr;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::*;

/// A lock-free stack of fibers.
pub struct Fibers {
  head: AtomicPtr<Node>,
}

struct Node {
  fiber: Box<Generator<Yield = (), Return = ()>>,
  next: *mut Node,
}

impl Fibers {
  /// Creates an empty `Fibers`.
  #[inline(always)]
  pub const fn new() -> Self {
    Self {
      head: AtomicPtr::new(ptr::null_mut()),
    }
  }

  pub(crate) fn add<G>(&self, gen: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: 'static,
  {
    self.push(Node::new(gen));
  }

  pub(crate) fn drain(&mut self) {
    let mut prev = ptr::null_mut();
    let mut curr = self.head.load(Acquire);
    while !curr.is_null() {
      unsafe {
        let next = (*curr).next;
        match (*curr).fiber.resume() {
          Yielded(()) => {
            prev = curr;
          }
          Complete(()) => {
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
        }
        curr = next;
      }
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
  #[inline(always)]
  fn new<G>(gen: G) -> Self
  where
    G: Generator<Yield = (), Return = ()>,
    G: 'static,
  {
    Self {
      fiber: Box::new(gen),
      next: ptr::null_mut(),
    }
  }
}
