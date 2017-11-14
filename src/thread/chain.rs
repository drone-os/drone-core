use core::ops::Generator;
use core::ops::GeneratorState::*;
use core::ptr;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::*;

/// A lock-free chain of routines.
pub struct Chain {
  head: AtomicPtr<Routine>,
}

struct Routine {
  routine: Box<Generator<Yield = (), Return = ()>>,
  next: *mut Routine,
}

impl Chain {
  /// Creates an empty `Chain`.
  #[inline(always)]
  pub const fn new() -> Self {
    Self {
      head: AtomicPtr::new(ptr::null_mut()),
    }
  }

  #[inline(always)]
  pub(crate) fn push<G>(&self, g: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: 'static,
  {
    let node = Box::into_raw(Box::new(Routine::new(g)));
    loop {
      let head = self.head.load(Relaxed);
      unsafe { (*node).next = head };
      if self.head.compare_and_swap(head, node, Release) == head {
        break;
      }
    }
  }

  #[inline(always)]
  pub(crate) fn drain(&mut self) {
    let mut prev = ptr::null_mut();
    let mut curr = self.head.load(Acquire);
    while !curr.is_null() {
      unsafe {
        let next = (*curr).next;
        match (*curr).routine.resume() {
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
}

impl Routine {
  #[inline(always)]
  fn new<G>(g: G) -> Self
  where
    G: Generator<Yield = (), Return = ()>,
    G: 'static,
  {
    Self {
      routine: Box::new(g),
      next: ptr::null_mut(),
    }
  }
}
