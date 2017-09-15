//! Routine handling.

use core::ops::Generator;
use core::ops::GeneratorState::*;
use core::ptr;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::*;
use prelude::*;

/// Linked list of routines.
pub struct RoutineList {
  ptr: AtomicPtr<Routine>,
}

/// Routine node.
pub struct Routine {
  thread: Box<Generator<Yield = (), Return = ()>>,
  next: RoutineList,
}

impl RoutineList {
  /// Constructs a vacant link.
  pub const fn vacant() -> Self {
    Self {
      ptr: AtomicPtr::new(ptr::null_mut()),
    }
  }

  /// Constructs an occupied link.
  pub const fn new(ptr: *mut Routine) -> Self {
    Self {
      ptr: AtomicPtr::new(ptr),
    }
  }

  /// Hardware invokes the routine chain with this method.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  pub unsafe fn invoke(&mut self) {
    let mut node = self;
    loop {
      let routine = node.ptr.load(Relaxed);
      if routine.is_null() {
        break;
      }
      match (*routine).thread.resume() {
        Yielded(()) => {
          node = &mut (*routine).next;
        }
        Complete(()) => {
          let next = (*routine).next.ptr.load(Relaxed);
          drop(Box::from_raw(routine));
          while node.ptr.compare_and_swap(routine, next, Relaxed) != routine {
            node = &mut (*node.ptr.load(Relaxed)).next;
          }
        }
      }
    }
  }

  /// Adds generator `g` first in the routine chain.
  pub fn push<G>(&mut self, g: G)
  where
    G: Generator<Yield = (), Return = ()> + Send + 'static,
  {
    let routine = Box::into_raw(Box::new(Routine::new(g)));
    loop {
      let current = self.ptr.load(Relaxed);
      unsafe {
        (*routine).next = RoutineList::new(current);
      }
      if self.ptr.compare_and_swap(current, routine, Relaxed) == current {
        break;
      }
    }
  }

  /// Adds closure `f` first in the routine chain.
  pub fn push_callback<F>(&mut self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    self.push(|| {
      if false {
        yield;
      }
      f()
    });
  }
}

impl Routine {
  /// Allocates a new routine thread.
  pub fn new<G>(g: G) -> Self
  where
    G: Generator<Yield = (), Return = ()> + Send + 'static,
  {
    Routine {
      thread: Box::new(g),
      next: RoutineList::vacant(),
    }
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
  fn size_of_routine_list() {
    assert_eq!(size_of::<RoutineList>(), size_of::<usize>());
  }

  #[test]
  fn generator() {
    let mut routine = RoutineList::vacant();
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
    let mut routine = RoutineList::vacant();
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
