use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

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

#[track_caller]
pub fn statemap_put(
    statemap: &'static BTreeMap<usize, BTreeMap<usize, AtomicUsize>>,
    key: usize,
    value: usize,
) {
    if let Some(map) = statemap.get(&key) {
        if let Some(counter) = map.get(&value) {
            counter.fetch_add(1, SeqCst);
        } else {
            panic!("incorrect state value {key} => {value}");
        }
    } else {
        panic!("incorrect state key {key} (=> {value})");
    }
}

#[allow(dead_code)]
#[track_caller]
pub fn statemap_put_counter(
    statemap: &'static BTreeMap<usize, BTreeMap<usize, AtomicUsize>>,
    counter: &'static AtomicUsize,
    key: usize,
) {
    statemap_put(statemap, key, counter.load(SeqCst));
}

#[track_caller]
pub fn statemap_check_exhaustive(
    rx_states: &'static BTreeMap<usize, BTreeMap<usize, AtomicUsize>>,
) {
    for (key, state) in rx_states {
        for (value, counter) in state {
            assert!(counter.load(SeqCst) != 0, "{key} => {value} not triggered");
        }
    }
}
