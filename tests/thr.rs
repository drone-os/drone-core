#![feature(generators)]

use drone_core::{
    fib, thr,
    thr::{prelude::*, Thread},
    token::Token,
};
use std::sync::{
    atomic::{AtomicI8, Ordering::*},
    Arc,
};

static mut THREADS: [Thr; 3] = [Thr::new(0), Thr::new(1), Thr::new(2)];

thr! {
    use THREADS;

    /// Test doc attribute
    #[doc = "test attribute"]
    pub struct Thr {
        #[allow(dead_code)]
        pub bar: isize = 1 - 2;
    }

    /// Test doc attribute
    #[doc = "test attribute"]
    pub struct ThrLocal {
        #[allow(dead_code)]
        pub foo: usize = 0;
    }
}

macro_rules! thr_num {
    ($name:ident, $position:expr) => {
        #[derive(Clone, Copy)]
        struct $name;

        unsafe impl Token for $name {
            unsafe fn take() -> Self {
                Self
            }
        }

        unsafe impl ThrToken for $name {
            type Thr = Thr;

            const THR_NUM: usize = $position;
        }
    };
}

thr_num!(Thr0, 0);
thr_num!(Thr1, 1);
thr_num!(Thr2, 2);

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
        let thr = Thr0::take();
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
        let thr = Thr1::take();
        thr.add_fn(move || {
            if inner.0.fetch_add(1, Relaxed) < 2 { fib::Yielded(()) } else { fib::Complete(()) }
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
fn fiber_once() {
    let counter = Arc::new(AtomicI8::new(0));
    let inner = Counter(Arc::clone(&counter));
    unsafe {
        let thr = Thr2::take();
        thr.add_once(move || {
            inner.0.fetch_add(1, Relaxed);
        });
        assert_eq!(counter.load(Relaxed), 0);
        thr.to_thr().fib_chain().drain();
        assert_eq!(counter.load(Relaxed), -2);
        thr.to_thr().fib_chain().drain();
        assert_eq!(counter.load(Relaxed), -2);
    }
}
