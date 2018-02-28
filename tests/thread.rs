#![feature(const_cell_new)]
#![feature(const_fn)]
#![feature(const_ptr_null_mut)]
#![feature(generators)]
#![feature(integer_atomics)]
#![feature(prelude_import)]
#![feature(proc_macro)]

extern crate drone_core;

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use drone_core::{fiber, thread};
use drone_core::thread::ThdToken;
use drone_core::thread::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering::*;

static mut THREADS: [Thd; 2] = [Thd::new(0), Thd::new(1)];

thread! {
  /// Test doc attribute
  #[doc = "test attribute"]
  pub struct Thd;
  extern static THREADS;

  #[allow(dead_code)]
  pub foo: usize = 0;
  #[allow(dead_code)]
  bar: isize = 1 - 2;
}

macro_rules! thread_number {
  ($name:ident, $position:expr) => {
    #[derive(Clone, Copy)]
    struct $name<T: ThdTag> {
      _tag: PhantomData<T>,
    }

    impl<T: ThdTag> $name<T> {
      unsafe fn new() -> Self {
        Self { _tag: PhantomData }
      }
    }

    impl<T: ThdTag> ThdToken<T> for $name<T> {
      type Thd = Thd;

      const THD_NUM: usize = $position;
    }

    impl<T: ThdTag> AsRef<Thd> for $name<T> {
      fn as_ref(&self) -> &Thd {
        self.as_thd()
      }
    }
  }
}

thread_number!(Thread0, 0);
thread_number!(Thread1, 1);

struct Counter(Arc<AtomicI8>);

impl Drop for Counter {
  fn drop(&mut self) {
    self.0.fetch_xor(0xFFu8 as i8, Relaxed);
  }
}

#[test]
fn fiber() {
  let counter = Arc::new(AtomicI8::new(0));
  let inner = Counter(Arc::clone(&counter));
  unsafe {
    let thread = Thread0::<Ltt>::new();
    fiber::spawn(thread, move || {
      while inner.0.fetch_add(1, Relaxed) < 2 {
        yield;
      }
    });
    assert_eq!(counter.load(Relaxed), 0);
    Thread0::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), 1);
    Thread0::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), 2);
    Thread0::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -4);
    Thread0::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -4);
  }
}

#[test]
fn fiber_fn() {
  let counter = Arc::new(AtomicI8::new(0));
  let inner = Counter(Arc::clone(&counter));
  unsafe {
    let thread = Thread1::<Ltt>::new();
    fiber::spawn_fn(thread, move || {
      inner.0.fetch_add(1, Relaxed);
    });
    assert_eq!(counter.load(Relaxed), 0);
    Thread1::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -2);
    Thread1::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -2);
  }
}
