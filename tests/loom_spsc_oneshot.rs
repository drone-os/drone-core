#[macro_use]
mod loom_spsc;

use std::pin::Pin;
use std::task::Poll;

use drone_core::sync::new_spsc::oneshot::{channel, Canceled};
use futures::future::FusedFuture;
use futures::prelude::*;

use self::loom_spsc::*;

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_drop() {
    loom::model(|| {
        let (tx, rx) = channel::<CheckDrop>();
        let tx = loom::thread::spawn(move || drop(tx));
        let rx = loom::thread::spawn(move || drop(rx));
        tx.join().unwrap();
        rx.join().unwrap();
    });
}

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_try_recv() {
    loom::model(|| {
        let (tx, mut rx) = channel::<CheckDrop>();
        let tx = loom::thread::spawn(move || drop(tx));
        let rx = loom::thread::spawn(move || match rx.try_recv() {
            Err(Canceled) | Ok(None) => {}
            value => panic!("{value:#?} variant is incorrect"),
        });
        tx.join().unwrap();
        rx.join().unwrap();
    });
}

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_recv() {
    let rx_states = statemap![0 => [101, 10100], 1 => [0, 10100]];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        let (tx, mut rx) = channel::<CheckDrop>();
        let tx = loom::thread::spawn(move || drop(tx));
        let rx = loom::thread::spawn(move || match Pin::new(&mut rx).poll(&mut rx_cx) {
            Poll::Pending => 0,
            Poll::Ready(Err(Canceled)) => {
                assert!(rx.is_terminated());
                1
            }
            Poll::Ready(Ok(_)) => {
                assert!(rx.is_terminated());
                2
            }
        });
        tx.join().unwrap();
        statemap_put(rx_states, rx_counter, rx.join().unwrap());
    });
    statemap_check_exhaustive(rx_states);
}

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_cancellation() {
    let tx_states = statemap![0 => [101, 10100], 1 => [0, 10100]];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, rx) = channel::<CheckDrop>();
        let rx = loom::thread::spawn(move || drop(rx));
        let tx =
            loom::thread::spawn(move || match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                Poll::Pending => 0,
                Poll::Ready(()) => {
                    assert!(tx.is_canceled());
                    1
                }
            });
        rx.join().unwrap();
        statemap_put(tx_states, tx_counter, tx.join().unwrap());
    });
    statemap_check_exhaustive(tx_states);
}

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_close_cancellation() {
    let tx_states = statemap![0 => [101, 10100], 1 => [0, 10100]];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, mut rx) = channel::<CheckDrop>();
        let rx = loom::thread::spawn(move || rx.close());
        let tx =
            loom::thread::spawn(move || match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                Poll::Pending => 0,
                Poll::Ready(()) => {
                    assert!(tx.is_canceled());
                    1
                }
            });
        rx.join().unwrap();
        statemap_put(tx_states, tx_counter, tx.join().unwrap());
    });
    statemap_check_exhaustive(tx_states);
}

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_send_recv() {
    let rx_states = statemap![
        10 => [101],
        12 => [0, 10100],
        20 => [10100],
    ];
    let data_states = statemap![
        10 => [1],
        12 => [10],
        20 => [10],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        let (tx, mut rx) = channel::<CheckDrop>();
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send(data) {
            Ok(()) => 10,
            Err(value) => {
                assert_eq!(value.get(10), 314);
                20
            }
        });
        let rx = loom::thread::spawn(move || match Pin::new(&mut rx).poll(&mut rx_cx) {
            Poll::Pending => 0,
            Poll::Ready(Err(Canceled)) => {
                assert!(rx.is_terminated());
                1
            }
            Poll::Ready(Ok(value)) => {
                assert!(rx.is_terminated());
                assert_eq!(value.get(10), 314);
                2
            }
        });
        let key = tx.join().unwrap() + rx.join().unwrap();
        statemap_put(rx_states, rx_counter, key);
        statemap_put(data_states, data_counter, key);
    });
    statemap_check_exhaustive(rx_states);
    statemap_check_exhaustive(data_states);
}

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_send_close_recv() {
    let rx_states = statemap![
        12 => [0],
        21 => [0],
    ];
    let data_states = statemap![
        12 => [10],
        21 => [10],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        let (tx, mut rx) = channel::<CheckDrop>();
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send(data) {
            Ok(()) => 10,
            Err(value) => {
                assert_eq!(value.get(10), 314);
                20
            }
        });
        let rx = loom::thread::spawn(move || {
            rx.close();
            match Pin::new(&mut rx).poll(&mut rx_cx) {
                Poll::Pending => 0,
                Poll::Ready(Err(Canceled)) => {
                    assert!(rx.is_terminated());
                    1
                }
                Poll::Ready(Ok(value)) => {
                    assert!(rx.is_terminated());
                    assert_eq!(value.get(10), 314);
                    2
                }
            }
        });
        let key = tx.join().unwrap() + rx.join().unwrap();
        statemap_put(rx_states, rx_counter, key);
        statemap_put(data_states, data_counter, key);
    });
    statemap_check_exhaustive(rx_states);
    statemap_check_exhaustive(data_states);
}

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_send_try_recv() {
    let data_states = statemap![
        10 => [1],
        12 => [10],
        20 => [10],
    ];
    loom::model(move || {
        let (tx, mut rx) = channel::<CheckDrop>();
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send(data) {
            Ok(()) => 10,
            Err(value) => {
                assert_eq!(value.get(10), 314);
                20
            }
        });
        let rx = loom::thread::spawn(move || match rx.try_recv() {
            Ok(None) => 0,
            Err(Canceled) => 1,
            Ok(Some(value)) => {
                assert_eq!(value.get(10), 314);
                2
            }
        });
        statemap_put(data_states, data_counter, tx.join().unwrap() + rx.join().unwrap());
    });
    statemap_check_exhaustive(data_states);
}

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_send_close_try_recv() {
    let data_states = statemap![
        12 => [10],
        21 => [10],
    ];
    loom::model(move || {
        let (tx, mut rx) = channel::<CheckDrop>();
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send(data) {
            Ok(()) => 10,
            Err(value) => {
                assert_eq!(value.get(10), 314);
                20
            }
        });
        let rx = loom::thread::spawn(move || {
            rx.close();
            match rx.try_recv() {
                Ok(None) => 0,
                Err(Canceled) => 1,
                Ok(Some(value)) => {
                    assert_eq!(value.get(10), 314);
                    2
                }
            }
        });
        statemap_put(data_states, data_counter, tx.join().unwrap() + rx.join().unwrap());
    });
    statemap_check_exhaustive(data_states);
}

#[cfg_attr(not(loom), ignore)]
#[test]
fn loom_recv_cancellation() {
    let tx_states = statemap![
        10 => [101, 10100],
        11 => [10100],
        20 => [10100],
    ];
    let rx_states = statemap![
        10 => [101, 10100],
        11 => [0, 10100],
        20 => [10100],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        async_context!(rx_counter, rx_waker, rx_cx);
        let (mut tx, mut rx) = channel::<CheckDrop>();
        let tx =
            loom::thread::spawn(move || match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                Poll::Pending => 10,
                Poll::Ready(()) => 20,
            });
        let rx = loom::thread::spawn(move || match Pin::new(&mut rx).poll(&mut rx_cx) {
            Poll::Pending => 0,
            Poll::Ready(Err(Canceled)) => 1,
            Poll::Ready(Ok(_)) => 2,
        });
        let key = tx.join().unwrap() + rx.join().unwrap();
        statemap_put(tx_states, tx_counter, key);
        statemap_put(rx_states, rx_counter, key);
    });
    statemap_check_exhaustive(tx_states);
    statemap_check_exhaustive(rx_states);
}
