#![feature(const_fn)]
#![feature(extern_in_paths)]
#![feature(generators)]
#![feature(never_type)]
#![feature(prelude_import)]
#![feature(proc_macro_gen)]
#![feature(proc_macro_path_invoc)]

#[macro_use]
extern crate drone_core;
extern crate futures;

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use drone_core::sync::spsc::oneshot;
use futures::prelude::*;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::*;
use std::sync::Arc;

static mut THREADS: [Thr; 1] = [Thr::new(0)];

struct Sv;

::drone_core::thr! {
  struct Thr;
  struct ThrLocal;
  extern struct Sv;
  extern static THREADS;
}

thread_local! {
  static COUNTER: Arc<Counter> = Arc::new(Counter(AtomicUsize::new(0)));
}

struct Counter(AtomicUsize);

impl task::Wake for Counter {
  fn wake(arc_self: &Arc<Self>) {
    arc_self.0.fetch_add(1, Relaxed);
  }
}

impl ::drone_core::sv::Supervisor for Sv {
  fn first() -> *const Self {
    ::std::ptr::null()
  }
}

#[test]
fn nested() {
  unsafe { drone_core::thr::init::<Thr>() };
  let (rx, tx) = oneshot::channel::<usize, !>();
  let mut fut = async(|| {
    await!(Box::new(async(|| await!(async(|| {
      let number = await!(rx)?;
      Ok::<usize, oneshot::RecvError<!>>(number + 1)
    })))))
  });
  COUNTER.with(|counter| {
    let waker = task::Waker::from(Arc::clone(counter));
    let mut map = task::LocalMap::new();
    let mut cx = task::Context::without_spawn(&mut map, &waker);
    counter.0.store(0, Relaxed);
    assert_eq!(fut.poll(&mut cx).unwrap(), Async::Pending);
    assert_eq!(fut.poll(&mut cx).unwrap(), Async::Pending);
    assert_eq!(counter.0.load(Relaxed), 0);
    assert_eq!(tx.send(Ok(1)), Ok(()));
    assert_eq!(counter.0.load(Relaxed), 1);
    assert_eq!(fut.poll(&mut cx).unwrap(), Async::Ready(2));
    assert_eq!(counter.0.load(Relaxed), 1);
  });
}
