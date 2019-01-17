#![feature(const_fn)]
#![feature(futures_api)]
#![feature(generators)]
#![feature(never_type)]
#![feature(prelude_import)]

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;
use drone_core::{awt, sv, sync::spsc::oneshot, thr};
use std::{
  future::Future,
  pin::Pin,
  ptr,
  sync::{
    atomic::{AtomicUsize, Ordering::*},
    Arc,
  },
  task::{local_waker_from_nonlocal, Poll, Wake},
};

static mut THREADS: [Thr; 1] = [Thr::new(0)];

struct Sv;

thr! {
  struct Thr;
  struct ThrLocal;
  extern struct Sv;
  extern static THREADS;
}

thread_local! {
  static COUNTER: Arc<Counter> = Arc::new(Counter(AtomicUsize::new(0)));
}

struct Counter(AtomicUsize);

impl Wake for Counter {
  fn wake(arc_self: &Arc<Self>) {
    arc_self.0.fetch_add(1, Relaxed);
  }
}

impl sv::Supervisor for Sv {
  fn first() -> *const Self {
    ptr::null()
  }
}

#[test]
fn nested() {
  unsafe { thr::init::<Thr>() };
  let (rx, tx) = oneshot::channel::<usize, !>();
  let mut fut = Box::pin(asnc(|| {
    awt!(Box::pin(asnc(|| awt!(asnc(|| {
      let number = awt!(rx)?;
      Ok::<usize, oneshot::RecvError<!>>(number + 1)
    })))))
  }));
  COUNTER.with(|counter| {
    let lw = local_waker_from_nonlocal(Arc::clone(counter));
    counter.0.store(0, Relaxed);
    assert_eq!(Pin::new(&mut fut).poll(&lw), Poll::Pending);
    assert_eq!(Pin::new(&mut fut).poll(&lw), Poll::Pending);
    assert_eq!(counter.0.load(Relaxed), 0);
    assert_eq!(tx.send(Ok(1)), Ok(()));
    assert_eq!(counter.0.load(Relaxed), 1);
    assert_eq!(Pin::new(&mut fut).poll(&lw), Poll::Ready(Ok(2)));
    assert_eq!(counter.0.load(Relaxed), 1);
  });
}
