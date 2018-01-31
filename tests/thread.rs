#![feature(const_cell_new)]
#![feature(const_fn)]
#![feature(const_ptr_null_mut)]
#![feature(generators)]
#![feature(integer_atomics)]
#![feature(prelude_import)]
#![feature(proc_macro)]

#[macro_use]
extern crate drone_core;
extern crate futures;

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use drone_core::thread::{thread_local, ThreadToken};
use drone_core::thread::prelude::*;
use futures::executor::{self, Notify};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering::*;

static mut THREADS: [ThreadLocal; 3] =
  [ThreadLocal::new(0), ThreadLocal::new(1), ThreadLocal::new(2)];

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

macro_rules! thread_number {
  ($name:ident, $position:expr) => {
    #[derive(Clone, Copy)]
    struct $name<T: ThreadTag> {
      _tag: PhantomData<T>,
    }

    impl<T: ThreadTag> $name<T> {
      unsafe fn new() -> Self {
        Self { _tag: PhantomData }
      }
    }

    impl<T: ThreadTag> ThreadToken<T> for $name<T> {
      type Thread = ThreadLocal;

      const THREAD_NUMBER: usize = $position;
    }

    impl<T: ThreadTag> Deref for $name<T> {
      type Target = ThreadLocal;

      fn deref(&self) -> &ThreadLocal {
        self.as_thread()
      }
    }
  }
}

thread_number!(Thread0, 0);
thread_number!(Thread1, 1);
thread_number!(Thread2, 2);

const NOTIFY_NOP: &NotifyNop = &NotifyNop;

struct Counter(Arc<AtomicI8>);

struct NotifyNop;

impl Drop for Counter {
  fn drop(&mut self) {
    self.0.fetch_xor(0xFFu8 as i8, Relaxed);
  }
}

impl Notify for NotifyNop {
  fn notify(&self, _: usize) {}
}

#[test]
fn fiber() {
  let counter = Arc::new(AtomicI8::new(0));
  let inner = Counter(Arc::clone(&counter));
  unsafe {
    let thread = Thread0::<Ltt>::new();
    thread.fiber(move || {
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
    thread.fiber_fn(move || {
      inner.0.fetch_add(1, Relaxed);
    });
    assert_eq!(counter.load(Relaxed), 0);
    Thread1::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -2);
    Thread1::<Ltt>::handler();
    assert_eq!(counter.load(Relaxed), -2);
  }
}

#[test]
fn fiber_future_scoped() {
  unsafe {
    let thread = Thread2::<Ltt>::new();
    let future = AsyncFuture::new(static || {
      let mut v = Vec::new();
      scoped_thread!(thread => scope {
        await!(scope.future(|| {
          for i in 0..3 {
            v.push(i);
            yield;
          }
          Ok(())
        }))
      })?;
      Ok::<_, ()>(v)
    });
    let mut executor = executor::spawn(future);
    let mut poll = || executor.poll_future_notify(&NOTIFY_NOP, 0);
    assert_eq!(poll(), Ok(Async::NotReady));
    Thread2::<Ltt>::handler();
    assert_eq!(poll(), Ok(Async::NotReady));
    Thread2::<Ltt>::handler();
    assert_eq!(poll(), Ok(Async::NotReady));
    Thread2::<Ltt>::handler();
    assert_eq!(poll(), Ok(Async::NotReady));
    Thread2::<Ltt>::handler();
    assert_eq!(poll(), Ok(Async::Ready(vec![0, 1, 2])));
  }
}
