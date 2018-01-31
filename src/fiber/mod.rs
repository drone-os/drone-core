//! Stackless semicoroutines.

mod future;
mod future_scoped;
mod stream_ring;
mod stream_unit;

pub use self::future::FiberFuture;
pub use self::future_scoped::FiberFutureScoped;
pub use self::stream_ring::FiberStreamRing;
pub use self::stream_unit::FiberStreamUnit;

use core::{mem, ptr};
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::*;

/// A lock-free stack of fibers.
pub struct Fibers {
  head: AtomicPtr<Node<'static>>,
}

struct Node<'scope> {
  fiber: Box<Generator<Yield = (), Return = ()> + 'scope>,
  next: *mut Node<'scope>,
}

impl Fibers {
  /// Creates an empty `Fibers`.
  #[inline(always)]
  pub const fn new() -> Self {
    Self {
      head: AtomicPtr::new(ptr::null_mut()),
    }
  }

  pub(crate) unsafe fn add_scoped<'scope, G>(&self, g: G)
  where
    G: Generator<Yield = (), Return = ()> + 'scope,
  {
    self.push(mem::transmute::<Node<'scope>, Node<'static>>(Node::new(g)));
  }

  pub(crate) fn add<G>(&self, g: G)
  where
    G: Generator<Yield = (), Return = ()> + 'static,
  {
    self.push(Node::new(g));
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

  fn push(&self, node: Node<'static>) {
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

impl<'scope> Node<'scope> {
  #[inline(always)]
  fn new<G>(g: G) -> Self
  where
    G: Generator<Yield = (), Return = ()> + 'scope,
  {
    Self {
      fiber: Box::new(g),
      next: ptr::null_mut(),
    }
  }
}
