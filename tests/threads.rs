#![feature(generators)]
#![no_implicit_prelude]

// A workaround for generators syntax. If we use `#![cfg(not(loom))]` for the
// whole test, the `generators` feature is not activated and the parser fails to
// parse `yield` keyword even that the code is not compiled.
#[cfg(not(loom))]
mod t {
    use ::drone_core::thr::prelude::*;
    use ::drone_core::thr::Thread;
    use ::drone_core::token::Token;
    use ::drone_core::{fib, thr};
    use ::std::assert_eq;
    use ::std::clone::Clone;
    use ::std::ops::Drop;
    use ::std::sync::atomic::AtomicI8;
    use ::std::sync::atomic::Ordering::*;
    use ::std::sync::Arc;

    thr::pool! {
        /// Test doc attribute
        #[doc = "test attribute"]
        thread => Thr {
            #[allow(dead_code)]
            pub bar: isize = 1 - 2;
        };

        /// Test doc attribute
        #[doc = "test attribute"]
        local => ThrLocal {
            #[allow(dead_code)]
            pub foo: usize = 0;
        };

        /// Test doc attribute
        #[doc = "test attribute"]
        index => Thrs;

        threads => {
            thr0;
            thr1;
            thr2;
        }
    }

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
}
