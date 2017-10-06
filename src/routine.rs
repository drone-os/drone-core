//! An interrupt service routine tasks chain.
//!
//! See [`Routine`] for more details.
//!
//! [`Routine`]: struct.Routine.html

use collections::LinkedList;
use core::ops::Generator;
use core::ops::GeneratorState::*;
use prelude::*;

/// An interrupt service routine tasks chain.
///
/// A lock-free data-structure to associate a task with an interrupt.
pub struct Routine {
  list: LinkedList<Node>,
}

struct Node {
  thread: Box<Generator<Yield = (), Return = ()>>,
}

impl<T> From<T> for Node
where
  T: Generator<Yield = (), Return = ()>,
  T: Send + 'static,
{
  #[inline]
  fn from(generator: T) -> Self {
    let thread = Box::new(generator);
    Self { thread }
  }
}

impl Routine {
  /// Creates an empty `Routine`.
  #[inline]
  pub const fn new() -> Self {
    Self {
      list: LinkedList::new(),
    }
  }

  /// The method is invoked by hardware to run the routine chain.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  pub unsafe fn invoke(&mut self) {
    self
      .list
      .drain_filter(|node| match node.thread.resume() {
        Yielded(()) => false,
        Complete(()) => true,
      })
      .for_each(|_| {});
  }

  /// Adds a generator first in the chain.
  pub fn push<G>(&mut self, g: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: Send + 'static,
  {
    self.list.push(g.into());
  }

  /// Adds a closure first in the chain.
  pub fn push_callback<F>(&mut self, f: F)
  where
    F: FnOnce(),
    F: Send + 'static,
  {
    self.push(|| {
      if false {
        yield;
      }
      f()
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use alloc::arc::Arc;
  use core::cell::Cell;
  use core::mem::size_of;

  struct Counter(Cell<i8>);

  struct Wrapper(Arc<Counter>);

  unsafe impl Sync for Counter {}

  impl Drop for Wrapper {
    fn drop(&mut self) {
      (self.0).0.set(-(self.0).0.get());
    }
  }

  #[test]
  fn size_of_routine() {
    assert_eq!(size_of::<Routine>(), size_of::<usize>());
  }

  #[test]
  fn generator() {
    let mut routine = Routine::new();
    let counter = Arc::new(Counter(Cell::new(0)));
    let wrapper = Wrapper(Arc::clone(&counter));
    routine.push(move || loop {
      {
        (wrapper.0).0.set((wrapper.0).0.get() + 1);
        if (wrapper.0).0.get() == 2 {
          break;
        }
      }
      yield;
    });
    assert_eq!(counter.0.get(), 0);
    unsafe {
      routine.invoke();
    }
    assert_eq!(counter.0.get(), 1);
    unsafe {
      routine.invoke();
    }
    assert_eq!(counter.0.get(), -2);
    unsafe {
      routine.invoke();
    }
    assert_eq!(counter.0.get(), -2);
  }

  #[test]
  fn callback() {
    let mut routine = Routine::new();
    let counter = Arc::new(Counter(Cell::new(0)));
    let wrapper = Wrapper(Arc::clone(&counter));
    routine.push_callback(move || {
      (wrapper.0).0.set((wrapper.0).0.get() + 1);
    });
    assert_eq!(counter.0.get(), 0);
    unsafe {
      routine.invoke();
    }
    assert_eq!(counter.0.get(), -1);
    unsafe {
      routine.invoke();
    }
    assert_eq!(counter.0.get(), -1);
  }
}
