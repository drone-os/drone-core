#![cfg(loom)]

#[macro_use]
mod loom_helpers;

use std::pin::Pin;
use std::task::Poll;

use drone_core::sync::spsc::ring::{channel, SendError, TryNextError, TrySendError};
use futures::prelude::*;
use futures::stream::FusedStream;

use self::loom_helpers::*;

#[test]
fn loom_drop() {
    loom::model(|| {
        let (tx, rx) = channel::<CheckDrop, CheckDrop>(2);
        let tx = loom::thread::spawn(move || drop(tx));
        let rx = loom::thread::spawn(move || drop(rx));
        tx.join().unwrap();
        rx.join().unwrap();
    });
}

#[test]
fn loom_try_next() {
    loom::model(|| {
        let (tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        let tx = loom::thread::spawn(move || drop(tx));
        let rx = loom::thread::spawn(move || match rx.try_next() {
            Err(_) => {}
            value => panic!("{value:#?} variant is incorrect"),
        });
        tx.join().unwrap();
        rx.join().unwrap();
    });
}

#[test]
fn loom_next() {
    let rx_states = statemap![
        0 => [0, 10100],
        3 => [101],
        6 => [101, 10100],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        let (tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        let tx = loom::thread::spawn(move || drop(tx));
        let rx = loom::thread::spawn(move || match Pin::new(&mut rx).poll_next(&mut rx_cx) {
            Poll::Ready(None) => {
                assert!(rx.is_terminated());
                0
            }
            Poll::Ready(Some(Ok(_))) => 1,
            Poll::Ready(Some(Err(_))) => 2,
            Poll::Pending => match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => {
                    assert!(rx.is_terminated());
                    3
                }
                Poll::Ready(Some(Ok(_))) => 4,
                Poll::Ready(Some(Err(_))) => 5,
                Poll::Pending => 6,
            },
        });
        tx.join().unwrap();
        statemap_put_counter(rx_states, rx_counter, rx.join().unwrap());
    });
    statemap_check_exhaustive(rx_states);
}

#[test]
fn loom_poll_close() {
    let tx_states = statemap![
        0 => [0, 10100],
        1 => [101],
        2 => [101, 10100],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, rx) = channel::<CheckDrop, CheckDrop>(2);
        let rx = loom::thread::spawn(move || drop(rx));
        let tx = loom::thread::spawn(move || match Pin::new(&mut tx).poll_close(&mut tx_cx) {
            Poll::Ready(Ok(())) => {
                assert!(tx.is_canceled());
                0
            }
            Poll::Pending => match Pin::new(&mut tx).poll_close(&mut tx_cx) {
                Poll::Ready(Ok(())) => {
                    assert!(tx.is_canceled());
                    1
                }
                Poll::Pending => 2,
                _ => panic!(),
            },
            _ => panic!(),
        });
        rx.join().unwrap();
        statemap_put_counter(tx_states, tx_counter, tx.join().unwrap());
    });
    statemap_check_exhaustive(tx_states);
}

#[test]
fn loom_poll_close_persistent() {
    let tx_states = statemap![
        0 => [0, 10100],
        1 => [101],
        2 => [101],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, rx) = channel::<CheckDrop, CheckDrop>(2);
        let rx = loom::thread::spawn(move || drop(rx));
        let tx = loom::thread::spawn(move || {
            let value = match Pin::new(&mut tx).poll_close(&mut tx_cx) {
                Poll::Ready(Ok(())) => {
                    assert!(tx.is_canceled());
                    0
                }
                Poll::Pending => match Pin::new(&mut tx).poll_close(&mut tx_cx) {
                    Poll::Ready(Ok(())) => {
                        assert!(tx.is_canceled());
                        1
                    }
                    Poll::Pending => 2,
                    _ => panic!(),
                },
                _ => panic!(),
            };
            (tx, tx_cx, value)
        });
        rx.join().unwrap();
        let (mut tx, mut tx_cx, mut tx_value) = tx.join().unwrap();
        if tx_value == 2 {
            tx_value = match Pin::new(&mut tx).poll_close(&mut tx_cx) {
                Poll::Ready(Ok(())) => {
                    assert!(tx.is_canceled());
                    2
                }
                Poll::Pending => 3,
                _ => panic!(),
            };
        }
        statemap_put_counter(tx_states, tx_counter, tx_value);
    });
    statemap_check_exhaustive(tx_states);
}

#[test]
fn loom_close_poll_close() {
    let tx_states = statemap![
        0 => [0, 10100],
        1 => [101],
        2 => [101, 10100],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        let rx = loom::thread::spawn(move || rx.close());
        let tx = loom::thread::spawn(move || match Pin::new(&mut tx).poll_close(&mut tx_cx) {
            Poll::Ready(Ok(())) => {
                assert!(tx.is_canceled());
                0
            }
            Poll::Pending => match Pin::new(&mut tx).poll_close(&mut tx_cx) {
                Poll::Ready(Ok(())) => {
                    assert!(tx.is_canceled());
                    1
                }
                Poll::Pending => 2,
                _ => panic!(),
            },
            _ => panic!(),
        });
        rx.join().unwrap();
        statemap_put_counter(tx_states, tx_counter, tx.join().unwrap());
    });
    statemap_check_exhaustive(tx_states);
}

#[test]
fn loom_close_poll_close_persistent() {
    let tx_states = statemap![
        0 => [0, 10100],
        1 => [101],
        2 => [101],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        let rx = loom::thread::spawn(move || {
            rx.close();
            rx
        });
        let tx = loom::thread::spawn(move || {
            let value = match Pin::new(&mut tx).poll_close(&mut tx_cx) {
                Poll::Ready(Ok(())) => {
                    assert!(tx.is_canceled());
                    0
                }
                Poll::Pending => match Pin::new(&mut tx).poll_close(&mut tx_cx) {
                    Poll::Ready(Ok(())) => {
                        assert!(tx.is_canceled());
                        1
                    }
                    Poll::Pending => 2,
                    _ => panic!(),
                },
                _ => panic!(),
            };
            (tx, tx_cx, value)
        });
        let rx = rx.join().unwrap();
        let (mut tx, mut tx_cx, mut tx_value) = tx.join().unwrap();
        if tx_value == 2 {
            tx_value = match Pin::new(&mut tx).poll_close(&mut tx_cx) {
                Poll::Ready(Ok(())) => {
                    assert!(tx.is_canceled());
                    2
                }
                Poll::Pending => 3,
                _ => panic!(),
            };
        }
        drop(rx);
        statemap_put_counter(tx_states, tx_counter, tx_value);
    });
    statemap_check_exhaustive(tx_states);
}

#[test]
fn loom_poll_flush() {
    let tx_states = statemap![0 => [0], 1 => [0]];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        let (mut tx, rx) = channel::<CheckDrop, CheckDrop>(2);
        let rx = loom::thread::spawn(move || drop(rx));
        let tx = loom::thread::spawn(move || match Pin::new(&mut tx).poll_flush(&mut tx_cx) {
            Poll::Ready(Ok(())) => 0,
            Poll::Ready(Err(SendError::Canceled)) => 1,
            Poll::Ready(Err(SendError::Full)) => 2,
            Poll::Pending => 3,
        });
        rx.join().unwrap();
        statemap_put_counter(tx_states, tx_counter, tx.join().unwrap());
    });
    statemap_check_exhaustive(tx_states);
}

#[test]
fn loom_send_err_next() {
    let rx_states = statemap![
        12 => [0, 10100],
        15 => [101],
        16 => [101],
        26 => [10100],
    ];
    let data_states = statemap![
        12 => [10],
        15 => [10],
        16 => [1],
        26 => [10],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        let (tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send_err(data) {
            Ok(()) => 10,
            Err(value) => {
                assert_eq!(value.get(10), 314);
                20
            }
        });
        let rx = loom::thread::spawn(move || match Pin::new(&mut rx).poll_next(&mut rx_cx) {
            Poll::Ready(None) => 0,
            Poll::Ready(Some(Ok(_))) => 1,
            Poll::Ready(Some(Err(value))) => {
                assert!(rx.is_terminated());
                assert_eq!(value.get(10), 314);
                2
            }
            Poll::Pending => match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 3,
                Poll::Ready(Some(Ok(_))) => 4,
                Poll::Ready(Some(Err(value))) => {
                    assert!(rx.is_terminated());
                    assert_eq!(value.get(10), 314);
                    5
                }
                Poll::Pending => 6,
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
fn loom_send_err_next_persistent() {
    let rx_states = statemap![
        12 => [0, 10100],
        15 => [101],
        18 => [101],
    ];
    let data_states = statemap![
        12 => [10],
        15 => [10],
        18 => [10],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        let (tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send_err(data) {
            Ok(()) => 10,
            Err(_) => 20,
        });
        let rx = loom::thread::spawn(move || {
            let value = match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 0,
                Poll::Ready(Some(Ok(_))) => 1,
                Poll::Ready(Some(Err(value))) => {
                    assert!(rx.is_terminated());
                    assert_eq!(value.get(10), 314);
                    2
                }
                Poll::Pending => match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                    Poll::Ready(None) => 3,
                    Poll::Ready(Some(Ok(_))) => 4,
                    Poll::Ready(Some(Err(value))) => {
                        assert!(rx.is_terminated());
                        assert_eq!(value.get(10), 314);
                        5
                    }
                    Poll::Pending => 6,
                },
            };
            (rx, rx_cx, value)
        });
        let tx_value = tx.join().unwrap();
        let (mut rx, mut rx_cx, mut rx_value) = rx.join().unwrap();
        if rx_value == 6 {
            rx_value = match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 6,
                Poll::Ready(Some(Ok(_))) => 7,
                Poll::Ready(Some(Err(value))) => {
                    assert!(rx.is_terminated());
                    assert_eq!(value.get(10), 314);
                    8
                }
                Poll::Pending => 9,
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
fn loom_feed_next_persistent() {
    let tx_states = statemap![
        1200 => [0],
        1210 => [0],
        1240 => [0],
        1420 => [0],
        1442 => [0],
    ];
    let rx_states = statemap![
        1200 => [0, 101, 10100],
        1210 => [0, 101, 10100],
        1240 => [101],
        1420 => [102],
        1442 => [102],
    ];
    let data_states = statemap![
        1200 => [10],
        1210 => [10],
        1240 => [10],
        1420 => [10],
        1442 => [10],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        async_context!(rx_counter, rx_waker, rx_cx);
        check_drop!(data_counter, data, 314);
        let (mut tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        let tx = loom::thread::spawn(move || match Pin::new(&mut tx.feed(data)).poll(&mut tx_cx) {
            Poll::Ready(Ok(())) => 1000,
            Poll::Ready(Err(_)) => 2000,
            Poll::Pending => 3000,
        });
        let rx = loom::thread::spawn(move || {
            let mut value = match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 100,
                Poll::Ready(Some(Ok(value))) => {
                    assert_eq!(value.get(10), 314);
                    200
                }
                Poll::Ready(Some(Err(_))) => 300,
                Poll::Pending => 400,
            };
            if !rx.is_terminated() {
                value += match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                    Poll::Ready(None) => 10,
                    Poll::Ready(Some(Ok(value))) => {
                        assert_eq!(value.get(10), 314);
                        20
                    }
                    Poll::Ready(Some(Err(_))) => 30,
                    Poll::Pending => 40,
                };
            }
            (rx, rx_cx, value)
        });
        let tx_value = tx.join().unwrap();
        let (mut rx, mut rx_cx, mut rx_value) = rx.join().unwrap();
        if !rx.is_terminated() {
            rx_value += match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 1,
                Poll::Ready(Some(Ok(value))) => {
                    assert_eq!(value.get(10), 314);
                    2
                }
                Poll::Ready(Some(Err(_))) => 3,
                Poll::Pending => 4,
            };
        }
        drop(rx);
        let key = tx_value + rx_value;
        statemap_put_counter(tx_states, tx_counter, key);
        statemap_put_counter(rx_states, rx_counter, key);
        statemap_put_counter(data_states, data_counter, key);
    });
    statemap_check_exhaustive(rx_states);
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_feed_send_err_next_persistent() {
    let tx_states = statemap![
        12300 => [0],
        12430 => [0],
        14230 => [0],
        14423 => [0],
    ];
    let rx_states = statemap![
        12300 => [0, 101, 10100],
        12430 => [101],
        14230 => [102],
        14423 => [102],
    ];
    let data_states = statemap![
        12300 => [10],
        12430 => [10],
        14230 => [10],
        14423 => [10],
    ];
    let err_states = statemap![
        12300 => [10],
        12430 => [10],
        14230 => [10],
        14423 => [10],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        async_context!(rx_counter, rx_waker, rx_cx);
        check_drop!(data_counter, data, 314);
        check_drop!(err_counter, err, 713);
        let (mut tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        let tx = loom::thread::spawn(move || match Pin::new(&mut tx.feed(data)).poll(&mut tx_cx) {
            Poll::Ready(Ok(())) => match tx.send_err(err) {
                Ok(()) => 10000,
                Err(_) => 20000,
            },
            Poll::Ready(Err(_)) => 30000,
            Poll::Pending => 40000,
        });
        let rx = loom::thread::spawn(move || {
            let mut value = match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 1000,
                Poll::Ready(Some(Ok(value))) => {
                    assert_eq!(value.get(10), 314);
                    2000
                }
                Poll::Ready(Some(Err(_))) => 3000,
                Poll::Pending => 4000,
            };
            if !rx.is_terminated() {
                value += match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                    Poll::Ready(None) => 100,
                    Poll::Ready(Some(Ok(value))) => {
                        assert_eq!(value.get(10), 314);
                        200
                    }
                    Poll::Ready(Some(Err(value))) => {
                        assert_eq!(value.get(10), 713);
                        300
                    }
                    Poll::Pending => 400,
                };
            }
            (rx, rx_cx, value)
        });
        let tx_value = tx.join().unwrap();
        let (mut rx, mut rx_cx, mut rx_value) = rx.join().unwrap();
        if !rx.is_terminated() {
            rx_value += match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 10,
                Poll::Ready(Some(Ok(value))) => {
                    assert_eq!(value.get(10), 314);
                    20
                }
                Poll::Ready(Some(Err(value))) => {
                    assert_eq!(value.get(10), 713);
                    30
                }
                Poll::Pending => 40,
            };
        }
        if !rx.is_terminated() {
            rx_value += match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 1,
                Poll::Ready(Some(Ok(value))) => {
                    assert_eq!(value.get(10), 314);
                    2
                }
                Poll::Ready(Some(Err(value))) => {
                    assert_eq!(value.get(10), 713);
                    3
                }
                Poll::Pending => 4,
            };
        }
        drop(rx);
        let key = tx_value + rx_value;
        statemap_put_counter(tx_states, tx_counter, key);
        statemap_put_counter(rx_states, rx_counter, key);
        statemap_put_counter(data_states, data_counter, key);
        statemap_put_counter(err_states, err_counter, key);
    });
    statemap_check_exhaustive(tx_states);
    statemap_check_exhaustive(rx_states);
    statemap_check_exhaustive(data_states);
    statemap_check_exhaustive(err_states);
}

#[test]
fn loom_send_next_persistent() {
    let tx_states = statemap![
        1200 => [10100],
        1210 => [0, 10100],
        1240 => [0, 10100],
        1420 => [0, 10100],
        3200 => [101, 10100, 10101],
        3210 => [101, 10101],
        3240 => [101, 10100, 10101],
        3420 => [101, 10100, 10101],
        3442 => [10100],
    ];
    let rx_states = statemap![
        1200 => [0, 101],
        1210 => [0, 101, 10100],
        1240 => [101],
        1420 => [102],
        3200 => [0, 101, 10100],
        3210 => [0, 101, 10100],
        3240 => [101],
        3420 => [102],
        3442 => [102],
    ];
    let data_states = statemap![
        1200 => [10],
        1210 => [10],
        1240 => [10],
        1420 => [10],
        3200 => [10],
        3210 => [10],
        3240 => [10],
        3420 => [10],
        3442 => [10],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        async_context!(rx_counter, rx_waker, rx_cx);
        check_drop!(data_counter, data, 314);
        let (mut tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        let tx = loom::thread::spawn(move || match Pin::new(&mut tx.send(data)).poll(&mut tx_cx) {
            Poll::Ready(Ok(())) => 1000,
            Poll::Ready(Err(_)) => 2000,
            Poll::Pending => 3000,
        });
        let rx = loom::thread::spawn(move || {
            let mut value = match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 100,
                Poll::Ready(Some(Ok(value))) => {
                    assert_eq!(value.get(10), 314);
                    200
                }
                Poll::Ready(Some(Err(_))) => 300,
                Poll::Pending => 400,
            };
            if !rx.is_terminated() {
                value += match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                    Poll::Ready(None) => 10,
                    Poll::Ready(Some(Ok(value))) => {
                        assert_eq!(value.get(10), 314);
                        20
                    }
                    Poll::Ready(Some(Err(_))) => 30,
                    Poll::Pending => 40,
                };
            }
            (rx, rx_cx, value)
        });
        let tx_value = tx.join().unwrap();
        let (mut rx, mut rx_cx, mut rx_value) = rx.join().unwrap();
        if !rx.is_terminated() {
            rx_value += match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => 1,
                Poll::Ready(Some(Ok(value))) => {
                    assert_eq!(value.get(10), 314);
                    2
                }
                Poll::Ready(Some(Err(_))) => 3,
                Poll::Pending => 4,
            };
        }
        drop(rx);
        let key = tx_value + rx_value;
        statemap_put_counter(tx_states, tx_counter, key);
        statemap_put_counter(rx_states, rx_counter, key);
        statemap_put_counter(data_states, data_counter, key);
    });
    statemap_check_exhaustive(rx_states);
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_send_err_close_next() {
    let rx_states = statemap![
        12 => [0],
        20 => [0],
    ];
    let data_states = statemap![
        12 => [10],
        20 => [10],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        let (tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send_err(data) {
            Ok(()) => 10,
            Err(value) => {
                assert_eq!(value.get(10), 314);
                20
            }
        });
        let rx = loom::thread::spawn(move || {
            rx.close();
            match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(None) => {
                    assert!(rx.is_terminated());
                    0
                }
                Poll::Ready(Some(Ok(_))) => 1,
                Poll::Ready(Some(Err(value))) => {
                    assert!(rx.is_terminated());
                    assert_eq!(value.get(10), 314);
                    2
                }
                Poll::Pending => 3,
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
fn loom_send_err_try_next() {
    let data_states = statemap![
        12 => [10],
        13 => [1],
        23 => [10],
    ];
    loom::model(move || {
        let (tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send_err(data) {
            Ok(()) => 10,
            Err(value) => {
                assert_eq!(value.get(10), 314);
                20
            }
        });
        let rx = loom::thread::spawn(move || match rx.try_next() {
            Err(TryNextError::Canceled) => 0,
            Ok(Ok(_)) => 1,
            Ok(Err(value)) => {
                assert_eq!(value.get(10), 314);
                2
            }
            Err(TryNextError::Empty) => 3,
        });
        statemap_put_counter(data_states, data_counter, tx.join().unwrap() + rx.join().unwrap());
    });
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_send_err_try_next_persistent() {
    let data_states = statemap![
        12 => [10],
        15 => [10],
    ];
    loom::model(move || {
        let (tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send_err(data) {
            Ok(()) => 10,
            Err(_) => 20,
        });
        let rx = loom::thread::spawn(move || {
            let value = match rx.try_next() {
                Err(TryNextError::Canceled) => 0,
                Ok(Ok(_)) => 1,
                Ok(Err(value)) => {
                    assert_eq!(value.get(10), 314);
                    2
                }
                Err(TryNextError::Empty) => 3,
            };
            (rx, value)
        });
        let tx_value = tx.join().unwrap();
        let (mut rx, mut rx_value) = rx.join().unwrap();
        if rx_value == 3 {
            rx_value = match rx.try_next() {
                Err(TryNextError::Canceled) => 3,
                Ok(Ok(_)) => 4,
                Ok(Err(value)) => {
                    assert_eq!(value.get(10), 314);
                    5
                }
                Err(TryNextError::Empty) => 6,
            };
        }
        statemap_put_counter(data_states, data_counter, tx_value + rx_value);
    });
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_send_err_close_try_next() {
    let data_states = statemap![
        12 => [10],
        20 => [10],
    ];
    loom::model(move || {
        let (tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        check_drop!(data_counter, data, 314);
        let tx = loom::thread::spawn(move || match tx.send_err(data) {
            Ok(()) => 10,
            Err(value) => {
                assert_eq!(value.get(10), 314);
                20
            }
        });
        let rx = loom::thread::spawn(move || {
            rx.close();
            match rx.try_next() {
                Err(TryNextError::Canceled) => 0,
                Ok(Ok(_)) => 1,
                Ok(Err(value)) => {
                    assert_eq!(value.get(10), 314);
                    2
                }
                Err(TryNextError::Empty) => 3,
            }
        });
        statemap_put_counter(data_states, data_counter, tx.join().unwrap() + rx.join().unwrap());
    });
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_next_poll_close() {
    let tx_states = statemap![
        13 => [101, 10100],
        10 => [10100],
        23 => [10100],
    ];
    let rx_states = statemap![
        13 => [101, 10100],
        10 => [0, 10100],
        23 => [10100],
    ];
    loom::model(move || {
        async_context!(tx_counter, tx_waker, tx_cx);
        async_context!(rx_counter, rx_waker, rx_cx);
        let (mut tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        let tx = loom::thread::spawn(move || match Pin::new(&mut tx).poll_close(&mut tx_cx) {
            Poll::Pending => 10,
            Poll::Ready(Ok(())) => 20,
            _ => panic!(),
        });
        let rx = loom::thread::spawn(move || match Pin::new(&mut rx).poll_next(&mut rx_cx) {
            Poll::Ready(None) => {
                assert!(rx.is_terminated());
                0
            }
            Poll::Ready(Some(Err(_))) => 1,
            Poll::Ready(Some(Ok(_))) => {
                assert!(rx.is_terminated());
                2
            }
            Poll::Pending => 3,
        });
        let key = tx.join().unwrap() + rx.join().unwrap();
        statemap_put_counter(tx_states, tx_counter, key);
        statemap_put_counter(rx_states, rx_counter, key);
    });
    statemap_check_exhaustive(tx_states);
    statemap_check_exhaustive(rx_states);
}

#[test]
fn loom_send_overwrite_next() {
    let rx_states = statemap![
        12 => [0],
        23 => [0],
        32 => [0],
        43 => [0],
    ];
    let data0_states = statemap![
        12 => [7],
        23 => [3],
        32 => [7],
        43 => [11],
    ];
    let data1_states = statemap![
        12 => [1],
        23 => [7],
        32 => [1],
        43 => [7],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        check_drops!(data_counters, data, [1, 3, 5]);
        let [value0, value1, value2]: [CheckDrop; 3] = data.try_into().unwrap();
        let (mut tx, mut rx) = channel::<CheckDrop, CheckDrop>(2);
        tx.try_send(value0).unwrap();
        tx.try_send(value1).unwrap();
        let tx = loom::thread::spawn(move || match tx.send_overwrite(value2) {
            Ok(None) => 10,
            Ok(Some(value)) => {
                assert_eq!(value.get(3), 1);
                20
            }
            Err((value, None)) => {
                assert_eq!(value.get(5), 5);
                30
            }
            Err((value, Some(overwritten))) => {
                assert_eq!(overwritten.get(11), 1);
                assert_eq!(value.get(5), 5);
                40
            }
        });
        let rx = loom::thread::spawn(move || match Pin::new(&mut rx).poll_next(&mut rx_cx) {
            Poll::Ready(None) => 0,
            Poll::Ready(Some(Err(_))) => 1,
            Poll::Ready(Some(Ok(value))) => {
                assert!(!rx.is_terminated());
                match value.get(7) {
                    1 => 2,
                    3 => 3,
                    _ => panic!(),
                }
            }
            Poll::Pending => 4,
        });
        let key = tx.join().unwrap() + rx.join().unwrap();
        statemap_put_counter(rx_states, rx_counter, key);
        statemap_put_counter(data0_states, data_counters[0], key);
        statemap_put_counter(data1_states, data_counters[1], key);
    });
    statemap_check_exhaustive(rx_states);
    statemap_check_exhaustive(data0_states);
    statemap_check_exhaustive(data1_states);
}

#[test]
fn loom_try_send_long_sequence_next_persistent() {
    let data_states = statemap![
        12 => [5, 7],
        22 => [5, 7],
        32 => [5],
        42 => [3],
        52 => [3],
        11 => [7],
        21 => [5, 7],
        31 => [5],
        41 => [3, 5],
        51 => [3, 5],
        10 => [7],
        20 => [7],
        30 => [5],
        40 => [5],
        50 => [5],
    ];
    loom::model(move || {
        async_context!(rx_counter, rx_waker, rx_cx);
        check_drops!(data_counters, data, [1, 3, 5, 7, 11]);
        let (mut tx, mut rx) = channel::<CheckDrop, CheckDrop>(3);
        let tx = loom::thread::spawn(move || {
            let mut remaining = Vec::new();
            for value in data {
                match tx.try_send(value) {
                    Ok(()) => {}
                    Err(TrySendError { err: SendError::Full, value }) => remaining.push(value),
                    Err(TrySendError { err: SendError::Canceled, .. }) => panic!(),
                }
            }
            remaining
        });
        let mut sum = 0;
        for _ in 0..2 {
            sum += match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(value) => value.unwrap().unwrap().get(7),
                Poll::Pending => 0,
            };
        }
        let remaining = tx.join().unwrap();
        let remaining_len = remaining.len();
        while !rx.is_terminated() {
            sum += match Pin::new(&mut rx).poll_next(&mut rx_cx) {
                Poll::Ready(value) => value.unwrap().unwrap().get(5),
                Poll::Pending => 0,
            };
        }
        for value in remaining {
            sum += value.get(3);
        }
        assert_eq!(sum, 27);
        for (i, counter) in data_counters.into_iter().enumerate() {
            statemap_put_counter(data_states, counter, (i + 1) * 10 + remaining_len);
        }
    });
    statemap_check_exhaustive(data_states);
}

#[test]
fn loom_try_send_long_sequence_try_next_persistent() {
    let data_states = statemap![
        12 => [5, 7],
        22 => [5, 7],
        32 => [5, 7],
        42 => [3],
        52 => [3],
        11 => [7],
        21 => [5, 7],
        31 => [5, 7],
        41 => [3, 5],
        51 => [3, 5],
        10 => [7],
        20 => [7],
        30 => [5, 7],
        40 => [5],
        50 => [5],
    ];
    loom::model(move || {
        check_drops!(data_counters, data, [1, 3, 5, 7, 11]);
        let (mut tx, mut rx) = channel::<CheckDrop, CheckDrop>(3);
        let tx = loom::thread::spawn(move || {
            let mut remaining = Vec::new();
            for value in data {
                match tx.try_send(value) {
                    Ok(()) => {}
                    Err(TrySendError { err: SendError::Full, value }) => remaining.push(value),
                    Err(TrySendError { err: SendError::Canceled, .. }) => panic!(),
                }
            }
            remaining
        });
        let mut sum = 0;
        for _ in 0..3 {
            sum += match rx.try_next() {
                Ok(value) => value.unwrap().get(7),
                Err(TryNextError::Empty) => 0,
                Err(TryNextError::Canceled) => panic!(),
            };
        }
        let remaining = tx.join().unwrap();
        let remaining_len = remaining.len();
        while !rx.is_terminated() {
            sum += match rx.try_next() {
                Ok(value) => value.unwrap().get(5),
                Err(TryNextError::Empty) => 0,
                Err(TryNextError::Canceled) => panic!(),
            };
        }
        for value in remaining {
            sum += value.get(3);
        }
        assert_eq!(sum, 27);
        for (i, counter) in data_counters.into_iter().enumerate() {
            statemap_put_counter(data_states, counter, (i + 1) * 10 + remaining_len);
        }
    });
    statemap_check_exhaustive(data_states);
}
