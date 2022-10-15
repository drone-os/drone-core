#![cfg(loom)]

#[macro_use]
mod loom_helpers;

use self::loom_helpers::*;
use drone_core::sync::LinkedList;

#[test]
fn loom_drop() {
    let data0_states = statemap![
        0 => [1],
    ];
    let data1_states = statemap![
        0 => [1],
    ];
    loom::model(|| {
        check_drops!(data_counters, data, [1, 3]);
        let [x, y]: [CheckDrop; 2] = data.try_into().unwrap();
        let list = LinkedList::new();
        assert!(list.is_empty());
        list.push(x);
        list.push(y);
        assert!(!list.is_empty());
        drop(list);
        statemap_put_counter(data0_states, data_counters[0], 0);
        statemap_put_counter(data1_states, data_counters[1], 0);
    });
    statemap_check_exhaustive(data0_states);
    statemap_check_exhaustive(data1_states);
}

#[test]
fn loom_push_push() {
    let data0_states = statemap![
        1 => [3],
        2 => [3],
    ];
    let data1_states = statemap![
        1 => [3],
        2 => [3],
    ];
    loom::model(|| {
        check_drops!(data_counters, data, [1, 3]);
        let [x, y]: [CheckDrop; 2] = data.try_into().unwrap();
        let list: &'static _ = Box::leak(Box::new(LinkedList::new()));
        assert!(list.is_empty());
        let x = loom::thread::spawn(move || list.push(x));
        let y = loom::thread::spawn(move || list.push(y));
        x.join().unwrap();
        y.join().unwrap();
        assert!(!list.is_empty());
        let x = list.pop().unwrap().get(3);
        let y = list.pop().unwrap().get(3);
        let key = match (x, y) {
            (1, 3) => 1,
            (3, 1) => 2,
            _ => 3,
        };
        statemap_put_counter(data0_states, data_counters[0], key);
        statemap_put_counter(data1_states, data_counters[1], key);
    });
    statemap_check_exhaustive(data0_states);
    statemap_check_exhaustive(data1_states);
}

#[test]
fn loom_push_pop() {
    let data_states = statemap![
        1 => [5],
        2 => [7],
    ];
    loom::model(|| {
        check_drop!(data_counter, data, 314);
        let list: &'static _ = Box::leak(Box::new(LinkedList::new()));
        let x = loom::thread::spawn(move || list.push(data));
        let y = loom::thread::spawn(move || list.pop());
        x.join().unwrap();
        let y = y.join().unwrap();
        let key = match y {
            Some(value) => {
                assert_eq!(value.get(5), 314);
                1
            }
            None => {
                assert_eq!(list.pop().unwrap().get(7), 314);
                2
            }
        };
        assert!(list.is_empty());
        statemap_put_counter(data_states, data_counter, key);
    });
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_pop_pop() {
    let data0_states = statemap![
        1 => [3],
        2 => [3],
    ];
    let data1_states = statemap![
        1 => [3],
        2 => [3],
    ];
    loom::model(|| {
        check_drops!(data_counters, data, [1, 3]);
        let [x, y]: [CheckDrop; 2] = data.try_into().unwrap();
        let list: &'static _ = Box::leak(Box::new(LinkedList::new()));
        assert!(list.is_empty());
        list.push(x);
        list.push(y);
        assert!(!list.is_empty());
        let x = loom::thread::spawn(move || list.pop());
        let y = loom::thread::spawn(move || list.pop());
        let x = x.join().unwrap().unwrap().get(3);
        let y = y.join().unwrap().unwrap().get(3);
        assert!(list.is_empty());
        assert!(list.pop().is_none());
        let key = match (x, y) {
            (1, 3) => 1,
            (3, 1) => 2,
            _ => 3,
        };
        statemap_put_counter(data0_states, data_counters[0], key);
        statemap_put_counter(data1_states, data_counters[1], key);
    });
    statemap_check_exhaustive(data0_states);
    statemap_check_exhaustive(data1_states);
}

#[test]
fn loom_drain_filter_raw() {
    let data0_states = statemap![
        0 => [5],
    ];
    let data1_states = statemap![
        0 => [5],
    ];
    let data2_states = statemap![
        0 => [3],
    ];
    loom::model(|| {
        check_drops!(data_counters, data, [1, 3, 5]);
        let [x, y, z]: [CheckDrop; 3] = data.try_into().unwrap();
        let list: &'static _ = Box::leak(Box::new(LinkedList::new()));
        list.push(x);
        list.push(y);
        let drained = loom::thread::spawn(move || unsafe {
            list.drain_filter_raw(|_| true).map(|x| Box::from_raw(x.cast_mut())).collect::<Vec<_>>()
        });
        let z = loom::thread::spawn(move || list.push(z));
        let mut drained = drained.join().unwrap();
        z.join().unwrap();
        assert_eq!(list.pop().unwrap().get(3), 5);
        assert_eq!(drained.pop().unwrap().value.get(5), 1);
        assert_eq!(drained.pop().unwrap().value.get(5), 3);
        statemap_put_counter(data0_states, data_counters[0], 0);
        statemap_put_counter(data1_states, data_counters[1], 0);
        statemap_put_counter(data2_states, data_counters[2], 0);
    });
    statemap_check_exhaustive(data0_states);
    statemap_check_exhaustive(data1_states);
    statemap_check_exhaustive(data2_states);
}

#[test]
fn loom_drain_filter_raw_drop() {
    let data0_states = statemap![
        0 => [1],
    ];
    let data1_states = statemap![
        0 => [1],
    ];
    loom::model(|| {
        check_drops!(data_counters, data, [1, 3]);
        let [x, y]: [CheckDrop; 2] = data.try_into().unwrap();
        let list: &'static _ = Box::leak(Box::new(LinkedList::new()));
        list.push(x);
        list.push(y);
        let drained = loom::thread::spawn(move || unsafe {
            list.drain_filter_raw(|_| true).for_each(|x| drop(Box::from_raw(x.cast_mut())))
        });
        drained.join().unwrap();
        assert!(list.is_empty());
        statemap_put_counter(data0_states, data_counters[0], 0);
        statemap_put_counter(data1_states, data_counters[1], 0);
    });
    statemap_check_exhaustive(data0_states);
    statemap_check_exhaustive(data1_states);
}
