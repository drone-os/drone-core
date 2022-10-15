use super::SoftThread;
use core::marker::PhantomData;
use core::task::{RawWaker, RawWakerVTable, Waker};

#[repr(transparent)]
pub struct SoftWaker<T: SoftThread> {
    thr_idx: u16,
    _marker: PhantomData<T>,
}

impl<T: SoftThread> SoftWaker<T> {
    pub fn new(thr_idx: u16) -> Self {
        Self { thr_idx, _marker: PhantomData }
    }

    pub fn wakeup(&self) {
        unsafe { T::set_pending(self.thr_idx) };
    }

    pub fn to_waker(&self) -> Waker {
        unsafe { Waker::from_raw(self.to_raw_waker()) }
    }

    fn to_raw_waker(&self) -> RawWaker {
        RawWaker::new(
            self.thr_idx as *const (),
            &RawWakerVTable::new(Self::clone, Self::wake, Self::wake, drop),
        )
    }

    unsafe fn clone(data: *const ()) -> RawWaker {
        Self::new(data as u16).to_raw_waker()
    }

    unsafe fn wake(data: *const ()) {
        Self::new(data as u16).wakeup();
    }
}
