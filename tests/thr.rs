#![feature(const_fn)]
#![feature(generators)]
#![feature(integer_atomics)]

extern crate drone_core;

use drone_core::{
  sv::Supervisor,
  thr::{prelude::*, ThrToken},
};
use std::{
  marker::PhantomData,
  ptr,
  sync::{
    atomic::{AtomicI8, Ordering::*},
    Arc,
  },
};

use drone_core::thr;

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

mod without_sv {
  use drone_core::thr;

  static mut THREADS: [Thr; 0] = [];

  thr! {
    struct Thr;
    struct ThrLocal;
    extern static THREADS;
  }
}

macro_rules! thr_num {
  ($name:ident, $position:expr) => {
    #[derive(Clone, Copy)]
    struct $name<T: ThrTag> {
      _tag: PhantomData<T>,
    }

    impl<T: ThrTag> ThrToken<T> for $name<T> {
      type Thr = Thr;
      type AThrToken = $name<Att>;
      type TThrToken = $name<Ttt>;
      type CThrToken = $name<Ctt>;
      type RThrToken = $name<Rtt>;

      const THR_NUM: usize = $position;

      unsafe fn take() -> Self {
        Self { _tag: PhantomData }
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
    let thr = Thr0::<Att>::take();
    thr.add(move || {
      while inner.0.fetch_add(1, Relaxed) < 2 {
        yield;
      }
    });
    assert_eq!(counter.load(Relaxed), 0);
    thr.to_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), 1);
    thr.to_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), 2);
    thr.to_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), -4);
    thr.to_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), -4);
  }
}

#[test]
fn fiber_fn() {
  let counter = Arc::new(AtomicI8::new(0));
  let inner = Counter(Arc::clone(&counter));
  unsafe {
    let thr = Thr1::<Att>::take();
    thr.add_fn(move || {
      inner.0.fetch_add(1, Relaxed);
    });
    assert_eq!(counter.load(Relaxed), 0);
    thr.to_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), -2);
    thr.to_thr().fib_chain().drain();
    assert_eq!(counter.load(Relaxed), -2);
  }
}
