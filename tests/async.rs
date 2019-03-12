#![feature(const_fn)]
#![feature(futures_api)]
#![feature(generators)]
#![feature(never_type)]
#![feature(prelude_import)]

use core::{
  pin::Pin,
  ptr,
  sync::atomic::{AtomicUsize, Ordering::*},
  task::{Poll, RawWaker, RawWakerVTable, Waker},
};
#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;
use drone_core::{awt, sv, sync::spsc::oneshot, thr};
use futures::prelude::*;

static mut THREADS: [Thr; 1] = [Thr::new(0)];

struct Sv;

thr! {
  struct Thr;
  struct ThrLocal;
  extern struct Sv;
  extern static THREADS;
}

struct Counter(AtomicUsize);

impl Counter {
  fn to_waker(&self) -> Waker {
    unsafe fn clone(counter: *const ()) -> RawWaker {
      RawWaker::new(counter, &VTABLE)
    }
    unsafe fn wake(counter: *const ()) {
      (*(counter as *const Counter)).0.fetch_add(1, Relaxed);
    }
    static VTABLE: RawWakerVTable = RawWakerVTable { clone, wake, drop };
    unsafe {
      Waker::new_unchecked(RawWaker::new(
        self as *const _ as *const (),
        &VTABLE,
      ))
    }
  }
}

impl sv::Supervisor for Sv {
  fn first() -> *const Self {
    ptr::null()
  }
}

#[test]
fn nested() {
  static COUNTER: Counter = Counter(AtomicUsize::new(0));
  unsafe { thr::init::<Thr>() };
  let (rx, tx) = oneshot::channel::<usize, !>();
  let mut fut = Box::pin(asnc(|| {
    awt!(Box::pin(asnc(|| {
      awt!(asnc(|| {
        let number = awt!(rx)?;
        Ok::<usize, oneshot::RecvError<!>>(number + 1)
      }))
    })))
  }));
  let waker = COUNTER.to_waker();
  COUNTER.0.store(0, Relaxed);
  assert_eq!(Pin::new(&mut fut).poll(&waker), Poll::Pending);
  assert_eq!(Pin::new(&mut fut).poll(&waker), Poll::Pending);
  assert_eq!(COUNTER.0.load(Relaxed), 0);
  assert_eq!(tx.send(Ok(1)), Ok(()));
  assert_eq!(COUNTER.0.load(Relaxed), 1);
  assert_eq!(Pin::new(&mut fut).poll(&waker), Poll::Ready(Ok(2)));
  assert_eq!(COUNTER.0.load(Relaxed), 1);
}
