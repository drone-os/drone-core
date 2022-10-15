#![cfg(loom)]

#[macro_use]
mod loom_helpers;

use self::loom_helpers::*;
use core::sync::atomic::Ordering::SeqCst;
use drone_core::sync::Mutex;
use futures::prelude::*;
use std::pin::Pin;
use std::task::Poll;

#[test]
fn loom_lock() {
    let data_states = statemap![
        12 => [3],
        13 => [3],
        21 => [3],
        31 => [3],
    ];
    let a_states = statemap![
        12 => [0],
        13 => [0],
        21 => [0, 10100],
        31 => [10100],
    ];
    let b_states = statemap![
        12 => [0, 10100],
        13 => [10100],
        21 => [0],
        31 => [0],
    ];
    loom::model(|| {
        async_context!(a_counter, a_waker, a_cx);
        async_context!(b_counter, b_waker, b_cx);
        check_drop!(data_counter, data, 314);
        let mutex: &'static _ = Box::leak(Box::new(Mutex::new(Some(data))));
        let a = loom::thread::spawn(move || match Pin::new(&mut mutex.lock()).poll(&mut a_cx) {
            Poll::Ready(mut guard) => match guard.take() {
                Some(value) => {
                    assert_eq!(value.get(3), 314);
                    10
                }
                None => 20,
            },
            Poll::Pending => 30,
        });
        let b = loom::thread::spawn(move || match Pin::new(&mut mutex.lock()).poll(&mut b_cx) {
            Poll::Ready(mut guard) => match guard.take() {
                Some(value) => {
                    assert_eq!(value.get(3), 314);
                    1
                }
                None => 2,
            },
            Poll::Pending => 3,
        });
        let a = a.join().unwrap();
        let b = b.join().unwrap();
        let key = a + b;
        statemap_put_counter(data_states, data_counter, key);
        statemap_put_counter(a_states, a_counter, key);
        statemap_put_counter(b_states, b_counter, key);
    });
    statemap_check_exhaustive(data_states);
    statemap_check_exhaustive(a_states);
    statemap_check_exhaustive(b_states);
}

#[test]
fn loom_lock_unlock() {
    let data_states = statemap![
        14 => [3],
        41 => [3],
    ];
    let a_states = statemap![
        14 => [0],
        41 => [101],
    ];
    let b_states = statemap![
        14 => [101],
        41 => [0],
    ];
    loom::model(|| {
        async_context!(a_counter, a_waker, a_cx);
        async_context!(b_counter, b_waker, b_cx);
        check_drop!(data_counter, data, 314);
        let mutex: &'static _ = Box::leak(Box::new(Mutex::new(Some(data))));
        let a = loom::thread::spawn(move || {
            let mut lock = mutex.lock();
            match Pin::new(&mut lock).poll(&mut a_cx) {
                Poll::Ready(mut guard) => Ok((
                    match guard.take() {
                        Some(value) => {
                            assert_eq!(value.get(3), 314);
                            10
                        }
                        None => 20,
                    },
                    guard,
                )),
                Poll::Pending => Err((lock, a_cx)),
            }
        });
        let b = loom::thread::spawn(move || {
            let mut lock = mutex.lock();
            match Pin::new(&mut lock).poll(&mut b_cx) {
                Poll::Ready(mut guard) => Ok((
                    match guard.take() {
                        Some(value) => {
                            assert_eq!(value.get(3), 314);
                            1
                        }
                        None => 2,
                    },
                    guard,
                )),
                Poll::Pending => Err((lock, b_cx)),
            }
        });
        let a = a.join().unwrap();
        let b = b.join().unwrap();
        assert!((!a.is_ok() || !b.is_ok()) && (a.is_ok() || b.is_ok()));
        assert!(a_counter.load(SeqCst) == 100 || b_counter.load(SeqCst) == 100);
        let a = a.map(|(a, _guard)| a);
        let b = b.map(|(b, _guard)| b);
        assert!(a_counter.load(SeqCst) == 101 || b_counter.load(SeqCst) == 101);
        let a =
            a.unwrap_or_else(|(mut lock, mut a_cx)| match Pin::new(&mut lock).poll(&mut a_cx) {
                Poll::Ready(mut guard) => match guard.take() {
                    Some(_) => 30,
                    None => 40,
                },
                Poll::Pending => 50,
            });
        let b =
            b.unwrap_or_else(|(mut lock, mut b_cx)| match Pin::new(&mut lock).poll(&mut b_cx) {
                Poll::Ready(mut guard) => match guard.take() {
                    Some(_) => 3,
                    None => 4,
                },
                Poll::Pending => 5,
            });
        let key = a + b;
        statemap_put_counter(data_states, data_counter, key);
        statemap_put_counter(a_states, a_counter, key);
        statemap_put_counter(b_states, b_counter, key);
    });
    statemap_check_exhaustive(data_states);
    statemap_check_exhaustive(a_states);
    statemap_check_exhaustive(b_states);
}

#[test]
fn loom_unlock_lock() {
    let data_states = statemap![
        13 => [3],
        31 => [3],
        33 => [0],
    ];
    let a_states = statemap![
        13 => [101, 10100, 10201],
        31 => [10100],
        33 => [101, 10100, 10201],
    ];
    let b_states = statemap![
        13 => [101, 10100, 10201],
        31 => [101, 10201],
        33 => [101, 10100, 10201],
    ];
    loom::model(|| {
        async_context!(a_counter, a_waker, a_cx);
        async_context!(b_counter, b_waker, b_cx);
        check_drop!(data_counter, data, 314);
        let mutex: &'static _ = Box::leak(Box::new(Mutex::new(Some(data))));
        let guard = mutex.try_lock().unwrap();
        let mut lock_a = mutex.lock();
        let mut lock_b = mutex.lock();
        assert!(matches!(Pin::new(&mut lock_a).poll(&mut a_cx), Poll::Pending));
        assert!(matches!(Pin::new(&mut lock_b).poll(&mut b_cx), Poll::Pending));
        let a = loom::thread::spawn(move || match Pin::new(&mut lock_a).poll(&mut a_cx) {
            Poll::Ready(mut guard) => (
                match guard.take() {
                    Some(value) => {
                        assert_eq!(value.get(3), 314);
                        10
                    }
                    None => 20,
                },
                Some(guard),
            ),
            Poll::Pending => (30, None),
        });
        let b = loom::thread::spawn(move || match Pin::new(&mut lock_b).poll(&mut b_cx) {
            Poll::Ready(mut guard) => (
                match guard.take() {
                    Some(value) => {
                        assert_eq!(value.get(3), 314);
                        1
                    }
                    None => 2,
                },
                Some(guard),
            ),
            Poll::Pending => (3, None),
        });
        drop(guard);
        let (a, guard1) = a.join().unwrap();
        let (b, guard2) = b.join().unwrap();
        drop((guard1, guard2));
        let key = a + b;
        statemap_put_counter(data_states, data_counter, key);
        statemap_put_counter(a_states, a_counter, key);
        statemap_put_counter(b_states, b_counter, key);
    });
    statemap_check_exhaustive(data_states);
    statemap_check_exhaustive(a_states);
    statemap_check_exhaustive(b_states);
}
