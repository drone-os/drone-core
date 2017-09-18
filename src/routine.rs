//! Routine handling.

use collections::LinkedList;
use core::ops::Generator;
use core::ops::GeneratorState::*;
use prelude::*;

/// A routine chain.
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
  /// Constructs an empty `Routine`.
  #[inline]
  pub const fn new() -> Self {
    Self {
      list: LinkedList::new(),
    }
  }

  /// Hardware invokes a routine chain with this method.
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

  /// Adds generator `g` first in the routine chain.
  pub fn push<G>(&mut self, g: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: Send + 'static,
  {
    self.list.push_front(g.into());
  }

  /// Adds closure `f` first in the routine chain.
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
  use core::cell::Cell;
  use core::mem::size_of;
  use core_alloc::arc::Arc;

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
