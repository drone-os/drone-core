mod wake;

use self::wake::SoftWaker;
use crate::thr::{AtomicOpaque, ThrExec, ThrToken, Thread};
use core::{
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
    task::Waker,
};

/// Software-managed thread.
///
/// TODO document implementation.
///
/// # Safety
///
/// TODO document `pending` size.
pub unsafe trait SoftThread: Thread {
    /// Number of priority levels.
    const PRIORITY_LEVELS: u8;

    /// Returns a reference to the opaque pending state storage.
    fn pending() -> &'static [AtomicOpaque<AtomicUsize>];

    /// Returns a storage for the current thread priority.
    fn current_priority() -> &'static AtomicOpaque<AtomicU8>;

    /// Reads the priority of the thread.
    fn priority(&self) -> &AtomicOpaque<AtomicU8>;

    /// Sets the `thr_idx` thread pending.
    ///
    /// See [the trait level documentation](SoftThread) for details.
    ///
    /// # Safety
    ///
    /// * `thr_idx` must be a valid index within [`Thread::threads`] array.
    /// * This function doesn't check for the thread token ownership.
    unsafe fn set_pending(thr_idx: usize);

    /// Sets the `thr_idx` thread pending and returns `true` if the thread
    /// priority is higher than the currently priority.
    ///
    /// If this function returned `true`, a subsequent call to
    /// [`SoftThread::preempt`] is needed.
    ///
    /// # Safety
    ///
    /// * `thr_idx` must be a valid index within [`Thread::threads`] array.
    /// * This function doesn't check for the thread token ownership.
    #[inline]
    unsafe fn will_preempt(thr_idx: usize) -> bool {
        // TODO set pending
        use core::cmp::Ordering::{Equal, Greater, Less};
        let thr = unsafe { Self::threads().get_unchecked(thr_idx).reveal() };
        let priority = thr.priority().reveal().load(Ordering::Relaxed);
        match priority.cmp(&Self::current_priority().reveal().load(Ordering::Relaxed)) {
            Less => true,
            Greater => false,
            Equal => thr_idx < Self::current().reveal().load(Ordering::Relaxed),
        }
    }

    /// Runs all pending threads with higher priorities than the current
    /// priority.
    fn preempt() {
        todo!()
    }
}

/// Token for a software-managed thread.
pub trait SoftThrToken: ThrToken {
    /// The software-managed thread type.
    type SoftThread: SoftThread;

    /// Returns a reference to the software-managed thread object.
    #[inline]
    fn to_soft_thr(self) -> &'static Self::SoftThread {
        unsafe { Self::SoftThread::threads().get_unchecked(Self::THR_IDX).reveal() }
    }

    /// Sets the thread pending.
    #[inline]
    fn set_pending(self) {
        unsafe { Self::SoftThread::set_pending(Self::THR_IDX) };
    }

    /// Clears the thread pending state.
    #[inline]
    fn clear_pending(self) {
        todo!()
    }

    /// Returns `true` if the thread is pending.
    #[inline]
    fn is_pending(self) -> bool {
        todo!()
    }

    /// Reads the priority of the thread.
    #[inline]
    fn priority(self) -> u8 {
        self.to_soft_thr().priority().reveal().load(Ordering::Relaxed)
    }

    /// Writes the priority of the thread.
    ///
    /// # Panics
    ///
    /// If `priority` is greater than or equals to
    /// `SoftThread::PRIORITY_LEVELS`.
    #[inline]
    fn set_priority(self, priority: u8) {
        assert!(priority < Self::SoftThread::PRIORITY_LEVELS);
        self.to_soft_thr().priority().reveal().store(priority, Ordering::Relaxed);
    }
}

impl<S: SoftThread, T: ThrToken<Thread = S>> SoftThrToken for T {
    type SoftThread = S;
}

impl<T: SoftThrToken> ThrExec for T {
    #[inline]
    fn wakeup(self) {
        SoftWaker::<T::SoftThread>::new(T::THR_IDX).wakeup();
    }

    #[inline]
    fn waker(self) -> Waker {
        SoftWaker::<T::SoftThread>::new(T::THR_IDX).to_waker()
    }
}
