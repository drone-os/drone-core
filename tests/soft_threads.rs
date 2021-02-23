use core::sync::atomic::{AtomicU8, AtomicUsize};
use drone_core::{
    thr,
    thr::{AtomicOpaque, SoftThread, ThrExec},
    token::Token,
};

thr::pool! {
    thread => Thr {
        priority: AtomicOpaque<AtomicU8> = AtomicOpaque::default_u8();
    };

    local => ThrLocal {};

    index => Thrs;

    threads => {
        thr_0;
        thr_1;
        thr_2;
    }
}

unsafe impl SoftThread for Thr {
    const PRIORITY_LEVELS: u8 = 8;

    fn pending() -> &'static [AtomicOpaque<AtomicUsize>] {
        const DEFAULT: AtomicOpaque<AtomicUsize> = AtomicOpaque::default_usize();
        static PENDING: [AtomicOpaque<AtomicUsize>; 2] = [DEFAULT; 2];
        &PENDING
    }

    fn current_priority() -> &'static AtomicOpaque<AtomicU8> {
        static CURRENT_PRIORITY: AtomicOpaque<AtomicU8> = AtomicOpaque::default_u8();
        &CURRENT_PRIORITY
    }

    fn priority(&self) -> &AtomicOpaque<AtomicU8> {
        &self.priority
    }

    unsafe fn set_pending(thr_idx: usize) {
        if Self::will_preempt(thr_idx) {
            Self::preempt();
        }
    }
}

#[test]
fn smoke() {
    let thr = unsafe { Thrs::take() };
    thr.thr_0.wakeup();
}
