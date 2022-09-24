use std::collections::BTreeMap;
use std::mem;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::task::{RawWaker, RawWakerVTable, Waker};

macro_rules! async_context {
    ($counter:ident, $waker:ident, $cx:ident) => {
        let $counter: &'static _ = Box::leak(Box::new(std::sync::atomic::AtomicUsize::new(0)));
        let $waker: &'static _ = Box::leak(Box::new(num_waker(&$counter)));
        let mut $cx = std::task::Context::from_waker(&$waker);
    };
}

macro_rules! statemap {
    ($($key:literal => [$($value:literal),*$(,)?]),*$(,)?) => {{
        #[allow(unused_mut)]
        let mut map = std::collections::BTreeMap::new();
        $(
            map.insert($key, {
                #[allow(unused_mut)]
                let mut inner = std::collections::BTreeMap::new();
                $(inner.insert($value, std::sync::atomic::AtomicUsize::new(0));)*
                inner
            });
        )*
        let map: &'static _ = Box::leak(Box::new(map));
        map
    }};
}

macro_rules! check_drop {
    ($counter:ident, $data:ident, $value:expr) => {
        let $counter: &'static _ = Box::leak(Box::new(std::sync::atomic::AtomicUsize::new(0)));
        let $data = CheckDrop($value, $counter);
    };
}

#[derive(Debug)]
pub struct CheckDrop(pub usize, pub &'static AtomicUsize);

impl CheckDrop {
    pub fn get(self, increment: usize) -> usize {
        let Self(value, atomic) = self;
        mem::forget(self);
        atomic.fetch_add(increment, SeqCst);
        value
    }
}

impl Drop for CheckDrop {
    fn drop(&mut self) {
        if self.1.fetch_add(1, SeqCst) > 0 {
            panic!("unexpected drop");
        }
    }
}

pub fn num_waker(num: &'static AtomicUsize) -> Waker {
    unsafe fn clone(counter: *const ()) -> RawWaker {
        unsafe { (*(counter as *const AtomicUsize)).fetch_add(100, SeqCst) };
        RawWaker::new(counter, &VTABLE)
    }
    unsafe fn wake(counter: *const ()) {
        unsafe { (*(counter as *const AtomicUsize)).fetch_add(1, SeqCst) };
    }
    fn drop(counter: *const ()) {
        unsafe { (*(counter as *const AtomicUsize)).fetch_add(10000, SeqCst) };
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);
    unsafe { Waker::from_raw(RawWaker::new(num as *const _ as *const (), &VTABLE)) }
}

pub fn statemap_put(
    statemap: &'static BTreeMap<usize, BTreeMap<usize, AtomicUsize>>,
    counter: &'static AtomicUsize,
    key: usize,
) {
    if let Some(map) = statemap.get(&key) {
        let value = counter.load(SeqCst);
        if let Some(counter) = map.get(&value) {
            counter.fetch_add(1, SeqCst);
        } else {
            panic!("incorrect state value {key} => {value}");
        }
    } else {
        panic!("incorrect state key {key}");
    }
}

pub fn statemap_check_exhaustive(
    rx_states: &'static BTreeMap<usize, BTreeMap<usize, AtomicUsize>>,
) {
    assert!(rx_states.values().all(|s| s.values().all(|c| c.load(SeqCst) > 0)));
}
