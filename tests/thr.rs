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

use drone_core::{fib, thr};
use drone_core::thr::ThrToken;
use drone_core::thr::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering::*;

static mut THREADS: [Thr; 2] = [Thr::new(0), Thr::new(1)];

thr! {
  /// Test doc attribute
  #[doc = "test attribute"]
  pub struct Thr;
  extern static THREADS;

  #[allow(dead_code)]
  pub foo: usize = 0;
  #[allow(dead_code)]
  bar: isize = 1 - 2;
}

macro_rules! thr_num {
  ($name:ident, $position:expr) => {
    #[derive(Clone, Copy)]
    struct $name<T: ThrTag> {
      _tag: PhantomData<T>,
    }

    impl<T: ThrTag> $name<T> {
      unsafe fn new() -> Self {
        Self { _tag: PhantomData }
      }
    }

    impl<T: ThrTag> ThrToken<T> for $name<T> {
      type Thr = Thr;

      const THR_NUM: usize = $position;
    }

    impl<T: ThrTag> AsRef<Thr> for $name<T> {
      fn as_ref(&self) -> &Thr {
        self.as_thr()
      }
    }
  }
}

thr_num!(Thr0, 0);
thr_num!(Thr1, 1);

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
    let thr = Thr0::<Ltt>::new();
    fib::spawn(thr, move || {
      while inner.0.fetch_add(1, Relaxed) < 2 {
        yield;
      }
    });
    assert_eq!(counter.load(Relaxed), 0);
    Thr0::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), 1);
    Thr0::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), 2);
    Thr0::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -4);
    Thr0::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -4);
  }
}

#[test]
fn fiber_fn() {
  let counter = Arc::new(AtomicI8::new(0));
  let inner = Counter(Arc::clone(&counter));
  unsafe {
    let thr = Thr1::<Ltt>::new();
    fib::spawn_fn(thr, move || {
      inner.0.fetch_add(1, Relaxed);
    });
    assert_eq!(counter.load(Relaxed), 0);
    Thr1::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -2);
    Thr1::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -2);
  }
}
