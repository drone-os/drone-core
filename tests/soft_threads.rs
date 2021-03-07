use core::sync::atomic::{AtomicU32, AtomicU8};
use drone_core::{
    thr,
    thr::{pending_size, SoftThread, ThrExec},
    token::Token,
};

thr::pool! {
    thread => Thr {
        priority: AtomicU8 = AtomicU8::new(0);
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
    fn pending() -> *const AtomicU32 {
        const VALUE: AtomicU32 = AtomicU32::new(0);
        const COUNT: usize = pending_size::<Thr>();
        static PENDING: [AtomicU32; COUNT] = [VALUE; COUNT];
        PENDING.as_ptr()
    }

    fn pending_priority() -> *const AtomicU8 {
        static PENDING_PRIORITY: AtomicU8 = AtomicU8::new(0);
        &PENDING_PRIORITY
    }

    fn priority(&self) -> *const AtomicU8 {
        &self.priority
    }

    unsafe fn set_pending(thr_idx: u16) {
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
