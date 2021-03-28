mod wake;

use self::wake::SoftWaker;
use crate::thr::{ThrExec, ThrToken, Thread};
use core::{
    sync::atomic::{AtomicU32, AtomicU8, Ordering},
    task::Waker,
};

/// Number of priority levels.
pub const PRIORITY_LEVELS: u8 = 27;

/// Returns the number of elements in [`SoftThread::pending`] array.
pub const fn pending_size<T: SoftThread>() -> usize {
    1 + pending_row_size::<T>() * PRIORITY_LEVELS as usize
}

const PRIORITY_MASK: u32 = u32::MAX << PRIORITY_LEVELS;

const fn pending_row_size<T: SoftThread>() -> usize {
    (T::COUNT / 32) as usize + (T::COUNT % 32 > 0) as usize
}

const fn pending_idx<T: SoftThread>(thr_idx: u16, priority: u8) -> usize {
    1 + pending_row_size::<T>() * (PRIORITY_LEVELS - 1 - priority) as usize
        + (thr_idx / 32) as usize
}

const fn pending_bit<T: SoftThread>(thr_idx: u16) -> u32 {
    1 << thr_idx % 32
}

/// Software-managed thread.
///
/// # Safety
///
/// [`SoftThread::pending`] must point to an array with [`pending_size`] number
/// of elements.
pub unsafe trait SoftThread: Thread {
    /// Returns a raw pointer to the pending state storage.
    fn pending() -> *const AtomicU32;

    /// Returns a raw pointer to the pending thread priority storage.
    fn pending_priority() -> *const AtomicU8;

    /// Returns a raw pointer to the thread priority storage.
    fn priority(&self) -> *const AtomicU8;

    /// Sets the `thr_idx` thread pending.
    ///
    /// See [the trait level documentation](SoftThread) for details.
    ///
    /// # Safety
    ///
    /// * `thr_idx` must be less than [`Thread::COUNT`].
    /// * This function doesn't check for the thread token ownership.
    #[inline]
    unsafe fn set_pending(thr_idx: u16) {
        if unsafe { Self::will_preempt(thr_idx) } {
            Self::preempt();
        }
    }

    /// Sets the `thr_idx` thread pending and returns `true` if the thread
    /// priority is higher than the currently priority.
    ///
    /// If this function returned `true`, a subsequent call to
    /// [`SoftThread::preempt`] is needed.
    ///
    /// # Safety
    ///
    /// * `thr_idx` must be less than [`Thread::COUNT`].
    /// * This function doesn't check for the thread token ownership.
    #[inline]
    unsafe fn will_preempt(thr_idx: u16) -> bool {
        unsafe {
            let mut priority =
                (*(*Self::pool().add(usize::from(thr_idx))).priority()).load(Ordering::Relaxed);
            let cursor = set_pending::<Self>(thr_idx, priority);
            priority += 1;
            cursor & pending_bit::<Self>(thr_idx) == 0
                && cursor >> PRIORITY_LEVELS < u32::from(priority)
                && (*Self::pending_priority()).fetch_max(priority, Ordering::Release) < priority
        }
    }

    /// Runs all pending threads with higher priorities than the current
    /// priority.
    #[inline]
    fn preempt() {
        unsafe {
            let mut priority = match (*Self::pending_priority()).swap(0, Ordering::Acquire) {
                0 => return,
                priority => u32::from(priority),
            };
            let prev_priority = match cursor_start::<Self>(priority) {
                Some(prev_priority) => prev_priority,
                None => return,
            };
            let mut ptr = Self::pending()
                .add(1 + pending_row_size::<Self>() * (PRIORITY_LEVELS - priority as u8) as usize);
            loop {
                let mut thr_idx = 0;
                'row: loop {
                    let mut cell = (*ptr).load(Ordering::Acquire);
                    if cell == 0 {
                        thr_idx += 32;
                        if thr_idx >= Self::COUNT {
                            break;
                        }
                    } else {
                        loop {
                            thr_resume::<Self>(&mut cell, ptr, thr_idx);
                            thr_idx += 1;
                            if thr_idx == Self::COUNT {
                                break 'row;
                            }
                            if thr_idx % 32 == 0 {
                                break;
                            }
                        }
                    }
                    ptr = ptr.add(1);
                }
                if cursor_advance::<Self>(&mut ptr, &mut priority, prev_priority) {
                    return;
                }
            }
        }
    }
}

/// Token for a software-managed thread.
pub trait SoftThrToken: ThrToken {
    /// The software-managed thread type.
    type SoftThread: SoftThread;

    /// Returns a reference to the software-managed thread object.
    #[inline]
    fn to_soft_thr(self) -> &'static Self::SoftThread {
        unsafe { &*Self::SoftThread::pool().add(usize::from(Self::THR_IDX)) }
    }

    /// Sets the thread pending.
    #[inline]
    fn set_pending(self) {
        unsafe { Self::SoftThread::set_pending(Self::THR_IDX) };
    }

    /// Clears the thread pending state.
    #[inline]
    fn clear_pending(self) {
        unsafe { clear_pending::<Self::SoftThread>(Self::THR_IDX, self.priority()) };
    }

    /// Returns `true` if the thread is pending.
    #[inline]
    fn is_pending(self) -> bool {
        unsafe { is_pending::<Self::SoftThread>(Self::THR_IDX, self.priority()) }
    }

    /// Reads the priority of the thread.
    #[inline]
    fn priority(self) -> u8 {
        unsafe { (*self.to_soft_thr().priority()).load(Ordering::Relaxed) }
    }

    /// Writes the priority of the thread.
    ///
    /// # Panics
    ///
    /// If `priority` is greater than or equals to [`PRIORITY_LEVELS`].
    #[inline]
    fn set_priority(self, priority: u8) {
        assert!(priority < PRIORITY_LEVELS);
        unsafe { (*self.to_soft_thr().priority()).store(priority, Ordering::Relaxed) };
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

unsafe fn cursor_start<T: SoftThread>(priority: u32) -> Option<u32> {
    unsafe {
        let mut prev_priority;
        let mut cursor = (*T::pending()).load(Ordering::Acquire);
        loop {
            prev_priority = cursor >> PRIORITY_LEVELS;
            if prev_priority >= priority {
                return None;
            }
            match (*T::pending()).compare_exchange_weak(
                cursor,
                (cursor & !PRIORITY_MASK | priority << PRIORITY_LEVELS) & !(1 << priority - 1),
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(prev_priority),
                Err(next_cursor) => cursor = next_cursor,
            }
        }
    }
}

unsafe fn cursor_advance<T: SoftThread>(
    ptr: &mut *const AtomicU32,
    priority: &mut u32,
    prev_priority: u32,
) -> bool {
    unsafe {
        let mut cursor = (*T::pending()).load(Ordering::Acquire);
        loop {
            if cursor & 1 << *priority - 1 == 0 {
                let mut next_priority = *priority;
                loop {
                    next_priority -= 1;
                    if next_priority == prev_priority {
                        match (*T::pending()).compare_exchange_weak(
                            cursor,
                            cursor & !PRIORITY_MASK | prev_priority << PRIORITY_LEVELS,
                            Ordering::Acquire,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => {
                                return true;
                            }
                            Err(next_cursor) => {
                                cursor = next_cursor;
                                break;
                            }
                        }
                    }
                    if cursor & 1 << next_priority - 1 != 0 {
                        match (*T::pending()).compare_exchange_weak(
                            cursor,
                            (cursor & !PRIORITY_MASK | next_priority << PRIORITY_LEVELS)
                                & !(1 << next_priority - 1),
                            Ordering::Acquire,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => {
                                *ptr = ptr.add(
                                    pending_row_size::<T>() * (*priority - next_priority) as usize,
                                );
                                *priority = next_priority;
                                return false;
                            }
                            Err(next_cursor) => {
                                cursor = next_cursor;
                                break;
                            }
                        }
                    }
                }
            } else {
                match (*T::pending()).compare_exchange_weak(
                    cursor,
                    cursor & !(1 << *priority - 1),
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return false,
                    Err(next_cursor) => cursor = next_cursor,
                }
            }
        }
    }
}

unsafe fn thr_resume<T: SoftThread>(cell: &mut u32, ptr: *const AtomicU32, thr_idx: u16) {
    unsafe {
        let thr_bit = pending_bit::<T>(thr_idx);
        if *cell & thr_bit == 0 {
            return;
        }
        loop {
            let next_cell = *cell & !thr_bit;
            match (*ptr).compare_exchange_weak(
                *cell,
                next_cell,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    *cell = next_cell;
                    break;
                }
                Err(next_cell) => {
                    *cell = next_cell;
                }
            }
        }
        T::call(thr_idx, T::resume);
    }
}

unsafe fn set_pending<T: SoftThread>(thr_idx: u16, priority: u8) -> u32 {
    unsafe {
        (*T::pending().add(pending_idx::<T>(thr_idx, priority)))
            .fetch_or(pending_bit::<T>(thr_idx), Ordering::Release);
        (*T::pending()).fetch_or(1 << priority, Ordering::Release)
    }
}

unsafe fn clear_pending<T: SoftThread>(thr_idx: u16, priority: u8) {
    unsafe {
        (*T::pending().add(pending_idx::<T>(thr_idx, priority)))
            .fetch_and(!pending_bit::<T>(thr_idx), Ordering::Relaxed);
    }
}

unsafe fn is_pending<T: SoftThread>(thr_idx: u16, priority: u8) -> bool {
    unsafe {
        (*T::pending().add(pending_idx::<T>(thr_idx, priority))).load(Ordering::Relaxed)
            & pending_bit::<T>(thr_idx)
            != 0
    }
}
