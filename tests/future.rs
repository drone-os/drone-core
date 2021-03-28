#![feature(generators)]
#![feature(never_type)]
#![feature(prelude_import)]
#![warn(unsafe_op_in_unsafe_fn)]

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use core::{
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering::*},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use drone_core::{sync::spsc::oneshot, thr};
use futures::prelude::*;

thr::pool! {
    thread => Thr {};
    local => ThrLocal {};
    index => Thrs;
    threads => { thread_0 };
}

struct Counter(AtomicUsize);

impl Counter {
    fn to_waker(&'static self) -> Waker {
        unsafe fn clone(counter: *const ()) -> RawWaker {
            RawWaker::new(counter, &VTABLE)
        }
        unsafe fn wake(counter: *const ()) {
            unsafe { (*(counter as *const Counter)).0.fetch_add(1, Relaxed) };
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);
        unsafe { Waker::from_raw(RawWaker::new(self as *const _ as *const (), &VTABLE)) }
    }
}

#[test]
fn test_await() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let waker = COUNTER.to_waker();
    let mut cx = Context::from_waker(&waker);
    let (tx, rx) = oneshot::channel::<usize>();
    let mut fut = Box::pin(async {
        let number = rx.await?;
        Ok::<usize, oneshot::Canceled>(number + 1)
    });
    assert_eq!(tx.send(1), Ok(()));
    assert_eq!(Pin::new(&mut fut).poll(&mut cx), Poll::Ready(Ok(2)));
}

#[test]
fn test_nested() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let waker = COUNTER.to_waker();
    let mut cx = Context::from_waker(&waker);
    let (tx, rx) = oneshot::channel::<usize>();
    let mut fut = Box::pin(async {
        Box::pin(async {
            async {
                let number = rx.await?;
                Ok::<usize, oneshot::Canceled>(number + 1)
            }
            .await
        })
        .await
    });
    assert_eq!(Pin::new(&mut fut).poll(&mut cx), Poll::Pending);
    assert_eq!(Pin::new(&mut fut).poll(&mut cx), Poll::Pending);
    assert_eq!(COUNTER.0.load(Relaxed), 0);
    assert_eq!(tx.send(1), Ok(()));
    assert_eq!(COUNTER.0.load(Relaxed), 1);
    assert_eq!(Pin::new(&mut fut).poll(&mut cx), Poll::Ready(Ok(2)));
    assert_eq!(COUNTER.0.load(Relaxed), 1);
}
