#![cfg(loom)]

#[macro_use]
mod loom_helpers;

use std::pin::Pin;
use std::task::Poll;

use drone_core::sync::spsc::oneshot::{channel, Canceled};
use futures::future::FusedFuture;
use futures::prelude::*;

use self::loom_helpers::*;

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

#[test]
fn loom_recv() {
    let rx_states = statemap![
        0 => [0, 10100],
        2 => [101],
        4 => [101, 10100],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        let (tx, mut rx) = channel::<CheckDrop>();
        let tx = loom::thread::spawn(move || drop(tx));
        let rx = loom::thread::spawn(move || match Pin::new(&mut rx).poll(&mut rx_cx) {
            Poll::Ready(Err(Canceled)) => {
                assert!(rx.is_terminated());
                0
            }
            Poll::Ready(Ok(_)) => 1,
            Poll::Pending => match Pin::new(&mut rx).poll(&mut rx_cx) {
                Poll::Ready(Err(Canceled)) => {
                    assert!(rx.is_terminated());
                    2
                }
                Poll::Ready(Ok(_)) => 3,
                Poll::Pending => 4,
            },
        });
        tx.join().unwrap();
        statemap_put_counter(rx_states, rx_counter, rx.join().unwrap());
    });
    statemap_check_exhaustive(rx_states);
}

#[test]
fn loom_cancellation() {
    let tx_states = statemap![
        0 => [0, 10100],
        1 => [101],
        2 => [101, 10100],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, rx) = channel::<CheckDrop>();
        let rx = loom::thread::spawn(move || drop(rx));
        let tx =
            loom::thread::spawn(move || match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                Poll::Ready(()) => {
                    assert!(tx.is_canceled());
                    0
                }
                Poll::Pending => match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                    Poll::Ready(()) => {
                        assert!(tx.is_canceled());
                        1
                    }
                    Poll::Pending => 2,
                },
            });
        rx.join().unwrap();
        statemap_put_counter(tx_states, tx_counter, tx.join().unwrap());
    });
    statemap_check_exhaustive(tx_states);
}

#[test]
fn loom_cancellation_persistent() {
    let tx_states = statemap![
        0 => [0, 10100],
        1 => [101],
        2 => [101],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, rx) = channel::<CheckDrop>();
        let rx = loom::thread::spawn(move || drop(rx));
        let tx = loom::thread::spawn(move || {
            let value = match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                Poll::Ready(()) => {
                    assert!(tx.is_canceled());
                    0
                }
                Poll::Pending => match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                    Poll::Ready(()) => {
                        assert!(tx.is_canceled());
                        1
                    }
                    Poll::Pending => 2,
                },
            };
            (tx, tx_cx, value)
        });
        rx.join().unwrap();
        let (mut tx, mut tx_cx, mut tx_value) = tx.join().unwrap();
        if tx_value == 2 {
            tx_value = match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                Poll::Ready(()) => {
                    assert!(tx.is_canceled());
                    2
                }
                Poll::Pending => 3,
            };
        }
        statemap_put_counter(tx_states, tx_counter, tx_value);
    });
    statemap_check_exhaustive(tx_states);
}

#[test]
fn loom_close_cancellation() {
    let tx_states = statemap![
        0 => [0, 10100],
        1 => [101],
        2 => [101, 10100],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, mut rx) = channel::<CheckDrop>();
        let rx = loom::thread::spawn(move || rx.close());
        let tx =
            loom::thread::spawn(move || match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                Poll::Ready(()) => {
                    assert!(tx.is_canceled());
                    0
                }
                Poll::Pending => match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                    Poll::Ready(()) => {
                        assert!(tx.is_canceled());
                        1
                    }
                    Poll::Pending => 2,
                },
            });
        rx.join().unwrap();
        statemap_put_counter(tx_states, tx_counter, tx.join().unwrap());
    });
    statemap_check_exhaustive(tx_states);
}

#[test]
fn loom_close_cancellation_persistent() {
    let tx_states = statemap![
        0 => [0, 10100],
        1 => [101],
        2 => [101],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, mut rx) = channel::<CheckDrop>();
        let rx = loom::thread::spawn(move || {
            rx.close();
            rx
        });
        let tx = loom::thread::spawn(move || {
            let value = match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                Poll::Ready(()) => {
                    assert!(tx.is_canceled());
                    0
                }
                Poll::Pending => match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                    Poll::Ready(()) => {
                        assert!(tx.is_canceled());
                        1
                    }
                    Poll::Pending => 2,
                },
            };
            (tx, tx_cx, value)
        });
        let rx = rx.join().unwrap();
        let (mut tx, mut tx_cx, mut tx_value) = tx.join().unwrap();
        if tx_value == 2 {
            tx_value = match Pin::new(&mut tx.cancellation()).poll(&mut tx_cx) {
                Poll::Ready(()) => {
                    assert!(tx.is_canceled());
                    2
                }
                Poll::Pending => 3,
            };
        }
        drop(rx);
        statemap_put_counter(tx_states, tx_counter, tx_value);
    });
    statemap_check_exhaustive(tx_states);
}

#[test]
fn loom_send_recv() {
    let rx_states = statemap![
        11 => [0, 10100],
        13 => [101],
        14 => [101],
        24 => [10100],
    ];
    let data_states = statemap![
        11 => [10],
        13 => [10],
        14 => [1],
        24 => [10],
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
            Poll::Ready(Err(Canceled)) => 0,
            Poll::Ready(Ok(value)) => {
                assert!(rx.is_terminated());
                assert_eq!(value.get(10), 314);
                1
            }
            Poll::Pending => match Pin::new(&mut rx).poll(&mut rx_cx) {
                Poll::Ready(Err(Canceled)) => 2,
                Poll::Ready(Ok(value)) => {
                    assert!(rx.is_terminated());
                    assert_eq!(value.get(10), 314);
                    3
                }
                Poll::Pending => 4,
            },
        });
        let key = tx.join().unwrap() + rx.join().unwrap();
        statemap_put_counter(rx_states, rx_counter, key);
        statemap_put_counter(data_states, data_counter, key);
    });
    statemap_check_exhaustive(rx_states);
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_send_recv_persistent() {
    let rx_states = statemap![
        11 => [0, 10100],
        13 => [101],
        15 => [101],
    ];
    let data_states = statemap![
        11 => [10],
        13 => [10],
        15 => [10],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        let (tx, mut rx) = channel::<CheckDrop>();
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send(data) {
            Ok(()) => 10,
            Err(_) => 20,
        });
        let rx = loom::thread::spawn(move || {
            let value = match Pin::new(&mut rx).poll(&mut rx_cx) {
                Poll::Ready(Err(Canceled)) => 0,
                Poll::Ready(Ok(value)) => {
                    assert!(rx.is_terminated());
                    assert_eq!(value.get(10), 314);
                    1
                }
                Poll::Pending => match Pin::new(&mut rx).poll(&mut rx_cx) {
                    Poll::Ready(Err(Canceled)) => 2,
                    Poll::Ready(Ok(value)) => {
                        assert!(rx.is_terminated());
                        assert_eq!(value.get(10), 314);
                        3
                    }
                    Poll::Pending => 4,
                },
            };
            (rx, rx_cx, value)
        });
        let tx_value = tx.join().unwrap();
        let (mut rx, mut rx_cx, mut rx_value) = rx.join().unwrap();
        if rx_value == 4 {
            rx_value = match Pin::new(&mut rx).poll(&mut rx_cx) {
                Poll::Ready(Err(Canceled)) => 4,
                Poll::Ready(Ok(value)) => {
                    assert!(rx.is_terminated());
                    assert_eq!(value.get(10), 314);
                    5
                }
                Poll::Pending => 6,
            };
        }
        let key = tx_value + rx_value;
        statemap_put_counter(rx_states, rx_counter, key);
        statemap_put_counter(data_states, data_counter, key);
    });
    statemap_check_exhaustive(rx_states);
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_send_close_recv() {
    let rx_states = statemap![
        11 => [0],
        20 => [0],
    ];
    let data_states = statemap![
        11 => [10],
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
        let rx = loom::thread::spawn(move || {
            rx.close();
            match Pin::new(&mut rx).poll(&mut rx_cx) {
                Poll::Ready(Err(Canceled)) => {
                    assert!(rx.is_terminated());
                    0
                }
                Poll::Ready(Ok(value)) => {
                    assert!(rx.is_terminated());
                    assert_eq!(value.get(10), 314);
                    1
                }
                Poll::Pending => 2,
            }
        });
        let key = tx.join().unwrap() + rx.join().unwrap();
        statemap_put_counter(rx_states, rx_counter, key);
        statemap_put_counter(data_states, data_counter, key);
    });
    statemap_check_exhaustive(rx_states);
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_send_try_recv() {
    let data_states = statemap![
        11 => [10],
        12 => [1],
        22 => [10],
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
            Err(Canceled) => 0,
            Ok(Some(value)) => {
                assert_eq!(value.get(10), 314);
                1
            }
            Ok(None) => 2,
        });
        statemap_put_counter(data_states, data_counter, tx.join().unwrap() + rx.join().unwrap());
    });
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_send_try_recv_persistent() {
    let data_states = statemap![
        11 => [10],
        13 => [10],
    ];
    loom::model(move || {
        let (tx, mut rx) = channel::<CheckDrop>();
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send(data) {
            Ok(()) => 10,
            Err(_) => 20,
        });
        let rx = loom::thread::spawn(move || {
            let value = match rx.try_recv() {
                Err(Canceled) => 0,
                Ok(Some(value)) => {
                    assert_eq!(value.get(10), 314);
                    1
                }
                Ok(None) => 2,
            };
            (rx, value)
        });
        let tx_value = tx.join().unwrap();
        let (mut rx, mut rx_value) = rx.join().unwrap();
        if rx_value == 2 {
            rx_value = match rx.try_recv() {
                Err(Canceled) => 2,
                Ok(Some(value)) => {
                    assert_eq!(value.get(10), 314);
                    3
                }
                Ok(None) => 4,
            };
        }
        statemap_put_counter(data_states, data_counter, tx_value + rx_value);
    });
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_send_close_try_recv() {
    let data_states = statemap![
        11 => [10],
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
        let rx = loom::thread::spawn(move || {
            rx.close();
            match rx.try_recv() {
                Err(Canceled) => 0,
                Ok(Some(value)) => {
                    assert_eq!(value.get(10), 314);
                    1
                }
                Ok(None) => 2,
            }
        });
        statemap_put_counter(data_states, data_counter, tx.join().unwrap() + rx.join().unwrap());
    });
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_recv_cancellation() {
    let tx_states = statemap![
        12 => [101, 10100],
        10 => [10100],
        22 => [10100],
    ];
    let rx_states = statemap![
        12 => [101, 10100],
        10 => [0, 10100],
        22 => [10100],
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
            Poll::Ready(Err(Canceled)) => {
                assert!(rx.is_terminated());
                0
            }
            Poll::Ready(Ok(_)) => 1,
            Poll::Pending => 2,
        });
        let key = tx.join().unwrap() + rx.join().unwrap();
        statemap_put_counter(tx_states, tx_counter, key);
        statemap_put_counter(rx_states, rx_counter, key);
    });
    statemap_check_exhaustive(tx_states);
    statemap_check_exhaustive(rx_states);
}
