#![feature(const_fn)]
#![feature(generators)]
#![feature(never_type)]
#![feature(prelude_import)]

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use core::{
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering::*},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use drone_core::{
    future::{self, fallback::*},
    sync::spsc::oneshot,
    thr,
};
use futures::prelude::*;

static mut THREADS: [Thr; 1] = [Thr::new(0)];

thr! {
    use THREADS;
    struct Thr {}
    struct ThrLocal {}
}

struct Counter(AtomicUsize);

impl Counter {
    fn to_waker(&'static self) -> Waker {
        unsafe fn clone(counter: *const ()) -> RawWaker {
            RawWaker::new(counter, &VTABLE)
        }
        unsafe fn wake(counter: *const ()) {
            (*(counter as *const Counter)).0.fetch_add(1, Relaxed);
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);
        unsafe { Waker::from_raw(RawWaker::new(self as *const _ as *const (), &VTABLE)) }
    }
}

fn test_awt() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let waker = COUNTER.to_waker();
    let mut cx = Context::from_waker(&waker);
    let (rx, tx) = oneshot::channel::<usize, !>();
    let mut fut = Box::pin(asyn(|| {
        let number = awt!(rx)?;
        Ok::<usize, oneshot::RecvError<!>>(number + 1)
    }));
    assert_eq!(tx.send(Ok(1)), Ok(()));
    assert_eq!(Pin::new(&mut fut).poll(&mut cx), Poll::Ready(Ok(2)));
}

fn test_nested() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let waker = COUNTER.to_waker();
    let mut cx = Context::from_waker(&waker);
    let (rx, tx) = oneshot::channel::<usize, !>();
    let mut fut = Box::pin(asyn(|| {
        awt!(Box::pin(asyn(|| {
            awt!(asyn(|| {
                let number = awt!(rx)?;
                Ok::<usize, oneshot::RecvError<!>>(number + 1)
            }))
        })))
    }));
    assert_eq!(Pin::new(&mut fut).poll(&mut cx), Poll::Pending);
    assert_eq!(Pin::new(&mut fut).poll(&mut cx), Poll::Pending);
    assert_eq!(COUNTER.0.load(Relaxed), 0);
    assert_eq!(tx.send(Ok(1)), Ok(()));
    assert_eq!(COUNTER.0.load(Relaxed), 1);
    assert_eq!(Pin::new(&mut fut).poll(&mut cx), Poll::Ready(Ok(2)));
    assert_eq!(COUNTER.0.load(Relaxed), 1);
}

// Tests that involves Drone threads shouldn't be run in parallel as the default
// test runner does. Therefore we wrap all tests into one test case.
#[test]
fn thread_sequence() {
    future::init::<Thr>();
    test_awt();
    test_nested();
}
