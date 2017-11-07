#![feature(const_cell_new)]
#![feature(const_fn)]
#![feature(const_ptr_null_mut)]
#![feature(decl_macro)]
#![feature(generators)]

extern crate drone;

use drone::thread::thread_local;
use std as core;
use std::sync::Arc;

static mut THREADS: [ThreadLocal; 1] = [ThreadLocal::new(0)];

thread_local! {
  //! Test doc attribute
  #![doc = "test attribute"]

  #[allow(dead_code)]
  pub foo: usize = { 0 }
  #[allow(dead_code)]
  bar: isize = { 1 - 2 }
}

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
    THREADS[0].routine(move || {
      loop {
        {
          (wrapper.0).0.set((wrapper.0).0.get() + 1);
          if (wrapper.0).0.get() == 2 {
            break;
          }
        }
        yield;
      }
    });
    assert_eq!(counter.0.get(), 0);
    THREADS[0].run(0);
    assert_eq!(counter.0.get(), 1);
    THREADS[0].run(0);
    assert_eq!(counter.0.get(), -2);
    THREADS[0].run(0);
    assert_eq!(counter.0.get(), -2);
  }
}

#[test]
fn routine_fn() {
  let counter = Arc::new(Counter(Cell::new(0)));
  let wrapper = Wrapper(Arc::clone(&counter));
  unsafe {
    THREADS[0].routine_fn(move || {
      (wrapper.0).0.set((wrapper.0).0.get() + 1);
    });
    assert_eq!(counter.0.get(), 0);
    THREADS[0].run(0);
    assert_eq!(counter.0.get(), -1);
    THREADS[0].run(0);
    assert_eq!(counter.0.get(), -1);
  }
}
