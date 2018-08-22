#![feature(const_fn)]
#![feature(extern_in_paths)]
#![feature(generators)]
#![feature(integer_atomics)]
#![feature(prelude_import)]
#![feature(proc_macro_gen)]

extern crate drone_core;

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use drone_core::sv::Supervisor;
use drone_core::thr::prelude::*;
use drone_core::thr::ThrToken;
use drone_core::{fib, thr};
use std::marker::PhantomData;
use std::ptr;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering::*;
use std::sync::Arc;

static mut THREADS: [Thr; 2] = [Thr::new(0), Thr::new(1)];

pub struct Sv;

impl Supervisor for Sv {
  fn first() -> *const Self {
    ptr::null()
  }
}

thr! {
  /// Test doc attribute
  #[doc = "test attribute"]
  pub struct Thr;
  /// Test doc attribute
  #[doc = "test attribute"]
  pub struct ThrLocal;
  extern struct Sv;
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
        Self::get_thr()
      }
    }
  };
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
    fib::add(thr, move || {
      while inner.0.fetch_add(1, Relaxed) < 2 {
        yield;
      }
    });
    assert_eq!(counter.load(Relaxed), 0);
    Thr0::<Ltt>::get_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), 1);
    Thr0::<Ltt>::get_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), 2);
    Thr0::<Ltt>::get_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), -4);
    Thr0::<Ltt>::get_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), -4);
  }
}

#[test]
fn fiber_fn() {
  let counter = Arc::new(AtomicI8::new(0));
  let inner = Counter(Arc::clone(&counter));
  unsafe {
    let thr = Thr1::<Ltt>::new();
    fib::add_fn(thr, move || {
      inner.0.fetch_add(1, Relaxed);
    });
    assert_eq!(counter.load(Relaxed), 0);
    Thr1::<Ltt>::get_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), -2);
    Thr1::<Ltt>::get_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), -2);
  }
}
