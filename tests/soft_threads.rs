#![no_implicit_prelude]

use ::drone_core::thr;
use ::drone_core::thr::{pending_size, SoftThrToken, SoftThread, ThrExec, PRIORITY_LEVELS};
use ::drone_core::token::Token;
use ::std::assert_eq;
use ::std::clone::Clone;
use ::std::sync::atomic::Ordering;
use ::std::sync::{Arc, Mutex};
use ::std::vec::Vec;

#[test]
fn test_set_pending() {
    thr::soft! {
        thread => Thr {};
        local => ThrLocal {};
        index => Thrs;
        threads => {};
        set_pending => set_pending;
    }
    unsafe fn set_pending(thr_idx: u16) {
        if Thr::will_preempt(thr_idx) {
            Thr::preempt();
        }
    }
}

#[test]
fn test_resume() {
    thr::soft! {
        thread => Thr {};
        local => ThrLocal {};
        index => Thrs;
        threads => {};
        resume => resume;
    }
    unsafe fn resume(thr: &Thr) {
        use ::drone_core::thr::prelude::*;
        thr.fib_chain().drain();
    }
}

#[test]
fn test_pending_size() {
    thr::soft! {
        thread => Thr0 {};
        local => ThrLocal0 {};
        index => Thrs0;
        threads => {};
    }
    thr::soft! {
        thread => Thr32 {};
        local => ThrLocal32 {};
        index => Thrs32;
        threads => {
            a0; a1; a2; a3; a4; a5; a6; a7; a8; a9; a10; a11; a12; a13; a14;
            a15; a16; a17; a18; a19; a20; a21; a22; a23; a24; a25; a26; a27;
            a28; a29; a30; a31;
        };
    }
    thr::soft! {
        thread => Thr33 {};
        local => ThrLocal33 {};
        index => Thrs33;
        threads => {
            b0; b1; b2; b3; b4; b5; b6; b7; b8; b9; b10; b11; b12; b13; b14;
            b15; b16; b17; b18; b19; b20; b21; b22; b23; b24; b25; b26; b27;
            b28; b29; b30; b31; b32;
        };
    }
    assert_eq!(pending_size::<Thr0>(), 1 + 0 * PRIORITY_LEVELS as usize);
    assert_eq!(pending_size::<Thr32>(), 1 + 1 * PRIORITY_LEVELS as usize);
    assert_eq!(pending_size::<Thr33>(), 1 + 2 * PRIORITY_LEVELS as usize);
}

#[test]
fn test_priorities() {
    thr::soft! {
        thread => Thr {};
        local => ThrLocal {};
        index => Thrs;
        threads => { thr_0; thr_1; thr_2; };
    }
    let Thrs { thr_0, thr_1, thr_2 } = unsafe { Thrs::take() };
    let log = Arc::new(Mutex::new(Vec::new()));
    let log_0 = Arc::clone(&log);
    let log_1 = Arc::clone(&log);
    let log_2 = Arc::clone(&log);
    thr_0.set_priority(0);
    thr_1.set_priority(1);
    thr_2.set_priority(2);
    thr_0.add_exec(async move {
        log_0.lock().unwrap().push(0);
        thr_2.wakeup();
        log_0.lock().unwrap().push(1);
    });
    thr_1.add_exec(async move {
        log_1.lock().unwrap().push(2);
    });
    thr_2.add_exec(async move {
        log_2.lock().unwrap().push(3);
        thr_1.wakeup();
        log_2.lock().unwrap().push(4);
    });
    thr_0.wakeup();
    assert_eq!(*log.lock().unwrap(), &[0, 3, 4, 2, 1]);
    assert_eq!(unsafe { &*Thr::pending_priority() }.load(Ordering::Relaxed), 0);
    for i in 0..pending_size::<Thr>() {
        assert_eq!(unsafe { &*Thr::pending().add(i) }.load(Ordering::Relaxed), 0);
    }
}
