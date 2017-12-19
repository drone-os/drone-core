#![feature(const_cell_new)]
#![feature(const_fn)]
#![feature(const_ptr_null_mut)]
#![feature(decl_macro)]
#![feature(generators)]
#![feature(prelude_import)]

extern crate drone_core;

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use drone_core::thread::thread_local;
use std::cell::Cell;
use std::ops::Deref;
use std::sync::Arc;

static mut THREADS: [ThreadLocal; 2] =
  [ThreadLocal::new(0), ThreadLocal::new(1)];

thread_local! {
  /// Test doc attribute
  #[doc = "test attribute"]
  ThreadLocal;
  THREADS;

  #[allow(dead_code)]
  pub foo: usize = { 0 }
  #[allow(dead_code)]
  bar: isize = { 1 - 2 }
}

macro_rules! thread_binding {
  ($name:ident, $index:expr) => {
    #[derive(Clone, Copy)]
    struct $name;

    impl ThreadBinding<ThreadLocal> for $name {
      const INDEX: usize = $index;

      unsafe fn bind() -> Self {
        $name
      }
    }

    impl Deref for $name {
      type Target = ThreadLocal;

      fn deref(&self) -> &ThreadLocal {
        self.as_thread()
      }
    }
  }
}

thread_binding!(Thread0, 0);
thread_binding!(Thread1, 1);

struct Counter(Cell<i8>);

struct Wrapper(Arc<Counter>);

unsafe impl Sync for Counter {}

impl Drop for Wrapper {
  fn drop(&mut self) {
    (self.0).0.set(-(self.0).0.get());
  }
}

#[test]
fn routine() {
  let counter = Arc::new(Counter(Cell::new(0)));
  let wrapper = Wrapper(Arc::clone(&counter));
  unsafe {
    THREADS[0].routine(move || loop {
      {
        (wrapper.0).0.set((wrapper.0).0.get() + 1);
        if (wrapper.0).0.get() == 2 {
          break;
        }
      }
      yield;
    });
    assert_eq!(counter.0.get(), 0);
    Thread0::handler();
    assert_eq!(counter.0.get(), 1);
    Thread0::handler();
    assert_eq!(counter.0.get(), -2);
    Thread0::handler();
    assert_eq!(counter.0.get(), -2);
  }
}

#[test]
fn routine_fn() {
  let counter = Arc::new(Counter(Cell::new(0)));
  let wrapper = Wrapper(Arc::clone(&counter));
  unsafe {
    THREADS[1].routine_fn(move || {
      (wrapper.0).0.set((wrapper.0).0.get() + 1);
    });
    assert_eq!(counter.0.get(), 0);
    Thread1::handler();
    assert_eq!(counter.0.get(), -1);
    Thread1::handler();
    assert_eq!(counter.0.get(), -1);
  }
}
