mod wake;

#[cfg(not(feature = "atomics"))]
use core::mem;
#[cfg(feature = "atomics")]
use core::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use core::task::Waker;

use self::wake::SoftWaker;
use crate::thr::{ThrExec, ThrToken, Thread};
#[cfg(not(feature = "atomics"))]
use crate::{platform::Interrupts, sync::soft_atomic::Atomic};

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

#[cfg(not(feature = "atomics"))]
type PendingState = Atomic<u32>;
#[cfg(feature = "atomics")]
type PendingState = AtomicU32;

#[cfg(not(feature = "atomics"))]
type PendingPriorityState = Atomic<u8>;
#[cfg(feature = "atomics")]
type PendingPriorityState = AtomicU8;

#[cfg(not(feature = "atomics"))]
type PriorityState = Atomic<u8>;
#[cfg(feature = "atomics")]
type PriorityState = AtomicU8;

/// Software-managed thread.
///
/// # Safety
///
/// [`SoftThread::pending`] must point to an array with [`pending_size`] number
/// of elements.
pub unsafe trait SoftThread: Thread {
    /// Returns a raw pointer to the pending state storage.
    fn pending() -> *const PendingState;

    /// Returns a raw pointer to the pending thread priority storage.
    fn pending_priority() -> *const PendingPriorityState;

    /// Returns a raw pointer to the thread priority storage.
    fn priority(&self) -> *const PriorityState;

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
        let thr_bit = pending_bit::<Self>(thr_idx);
        #[cfg(not(feature = "atomics"))]
        unsafe {
            let mut priority = (*(*Self::pool().add(usize::from(thr_idx))).priority()).load();
            let cell_ptr = Self::pending().add(pending_idx::<Self>(thr_idx, priority));
            let _critical = Interrupts::enter();
            let cell = (*cell_ptr).load();
            if cell & thr_bit != 0 {
                return false;
            }
            (*cell_ptr).store(cell | thr_bit);
            let cursor = (*Self::pending()).load();
            (*Self::pending()).store(cursor | 1 << priority);
            priority += 1;
            if cursor >> PRIORITY_LEVELS >= u32::from(priority) {
                return false;
            }
            let pending_priority = (*Self::pending_priority()).load();
            if pending_priority >= priority {
                return false;
            }
            (*Self::pending_priority()).store(priority);
            true
        }
        #[cfg(feature = "atomics")]
        unsafe {
            let mut priority =
                (*(*Self::pool().add(usize::from(thr_idx))).priority()).load(Ordering::Relaxed);
            let cell_ptr = Self::pending().add(pending_idx::<Self>(thr_idx, priority));
            if (*cell_ptr).fetch_or(thr_bit, Ordering::Release) & thr_bit != 0 {
                return false;
            }
            let cursor = (*Self::pending()).fetch_or(1 << priority, Ordering::Release);
            priority += 1;
            cursor >> PRIORITY_LEVELS < u32::from(priority)
                && (*Self::pending_priority()).fetch_max(priority, Ordering::Release) < priority
        }
    }

    /// Runs all pending threads with higher priorities than the current
    /// priority.
    #[inline]
    fn preempt() {
        unsafe {
            let (mut priority, prev_priority) = match cursor_claim::<Self>() {
                Some((priority, prev_priority)) => (priority, prev_priority),
                None => return,
            };
            let mut cell_ptr = Self::pending()
                .add(1 + pending_row_size::<Self>() * (PRIORITY_LEVELS - priority as u8) as usize);
            loop {
                let mut thr_idx = 0;
                'row: loop {
                    #[cfg(not(feature = "atomics"))]
                    let mut loaded_cell = (*cell_ptr).load();
                    #[cfg(feature = "atomics")]
                    let mut loaded_cell = (*cell_ptr).load(Ordering::Acquire);
                    if loaded_cell == 0 {
                        thr_idx += 32;
                        if thr_idx >= Self::COUNT {
                            break;
                        }
                    } else {
                        loop {
                            check_resume::<Self>(&mut loaded_cell, cell_ptr, thr_idx);
                            thr_idx += 1;
                            if thr_idx == Self::COUNT {
                                break 'row;
                            }
                            if thr_idx % 32 == 0 {
                                break;
                            }
                        }
                    }
                    cell_ptr = cell_ptr.add(1);
                }
                if cursor_advance::<Self>(&mut cell_ptr, &mut priority, prev_priority) {
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
        let thr_idx = pending_idx::<Self::SoftThread>(Self::THR_IDX, self.priority());
        let thr_mask = !pending_bit::<Self::SoftThread>(Self::THR_IDX);
        #[cfg(not(feature = "atomics"))]
        unsafe {
            (*Self::SoftThread::pending().add(thr_idx)).modify(|cell| cell & thr_mask);
        }
        #[cfg(feature = "atomics")]
        unsafe {
            (*Self::SoftThread::pending().add(thr_idx)).fetch_and(thr_mask, Ordering::Relaxed);
        }
    }

    /// Returns `true` if the thread is pending.
    #[inline]
    fn is_pending(self) -> bool {
        let thr_idx = pending_idx::<Self::SoftThread>(Self::THR_IDX, self.priority());
        let thr_bit = pending_bit::<Self::SoftThread>(Self::THR_IDX);
        #[cfg(not(feature = "atomics"))]
        unsafe {
            (*Self::SoftThread::pending().add(thr_idx)).load() & thr_bit != 0
        }
        #[cfg(feature = "atomics")]
        unsafe {
            (*Self::SoftThread::pending().add(thr_idx)).load(Ordering::Relaxed) & thr_bit != 0
        }
    }

    /// Reads the priority of the thread.
    #[inline]
    fn priority(self) -> u8 {
        let current_priority = self.to_soft_thr().priority();
        #[cfg(not(feature = "atomics"))]
        unsafe {
            (*current_priority).load()
        }
        #[cfg(feature = "atomics")]
        unsafe {
            (*current_priority).load(Ordering::Relaxed)
        }
    }

    /// Writes the priority of the thread.
    ///
    /// # Panics
    ///
    /// If `priority` is greater than or equals to [`PRIORITY_LEVELS`].
    #[inline]
    fn set_priority(self, priority: u8) {
        assert!(priority < PRIORITY_LEVELS);
        let current_priority = self.to_soft_thr().priority();
        #[cfg(not(feature = "atomics"))]
        unsafe {
            (*current_priority).store(priority);
        }
        #[cfg(feature = "atomics")]
        unsafe {
            (*current_priority).store(priority, Ordering::Relaxed);
        }
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

unsafe fn cursor_claim<T: SoftThread>() -> Option<(u32, u32)> {
    #[cfg(not(feature = "atomics"))]
    unsafe {
        let _critical = Interrupts::enter();
        let pending_priority = mem::replace(&mut *(*T::pending_priority()).as_mut_ptr(), 0);
        let priority = match pending_priority {
            0 => return None,
            priority => u32::from(priority),
        };
        let cursor = (*T::pending()).load();
        let prev_priority = cursor >> PRIORITY_LEVELS;
        if prev_priority >= priority {
            return None;
        }
        (*T::pending())
            .store((cursor & !PRIORITY_MASK | priority << PRIORITY_LEVELS) & !(1 << priority - 1));
        Some((priority, prev_priority))
    }
    #[cfg(feature = "atomics")]
    unsafe {
        let priority = match (*T::pending_priority()).swap(0, Ordering::Acquire) {
            0 => return None,
            priority => u32::from(priority),
        };
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
                Ok(_) => return Some((priority, prev_priority)),
                Err(next_cursor) => cursor = next_cursor,
            }
        }
    }
}

unsafe fn cursor_advance<T: SoftThread>(
    cell_ptr: &mut *const PendingState,
    priority: &mut u32,
    prev_priority: u32,
) -> bool {
    #[cfg(not(feature = "atomics"))]
    unsafe {
        let _critical = Interrupts::enter();
        let cursor = (*T::pending()).load();
        if cursor & 1 << *priority - 1 == 0 {
            let mut next_priority = *priority;
            loop {
                next_priority -= 1;
                if next_priority == prev_priority {
                    (*T::pending())
                        .store(cursor & !PRIORITY_MASK | prev_priority << PRIORITY_LEVELS);
                    return true;
                }
                if cursor & 1 << next_priority - 1 != 0 {
                    (*T::pending()).store(
                        (cursor & !PRIORITY_MASK | next_priority << PRIORITY_LEVELS)
                            & !(1 << next_priority - 1),
                    );
                    *cell_ptr = cell_ptr
                        .add(pending_row_size::<T>() * (*priority - next_priority) as usize);
                    *priority = next_priority;
                    return false;
                }
            }
        } else {
            (*T::pending()).store(cursor & !(1 << *priority - 1));
            false
        }
    }
    #[cfg(feature = "atomics")]
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
                                *cell_ptr = cell_ptr.add(
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

unsafe fn check_resume<T: SoftThread>(
    loaded_cell: &mut u32,
    cell_ptr: *const PendingState,
    thr_idx: u16,
) {
    let thr_bit = pending_bit::<T>(thr_idx);
    if *loaded_cell & thr_bit == 0 {
        return;
    }
    #[cfg(not(feature = "atomics"))]
    Interrupts::section(|| unsafe {
        *loaded_cell = (*cell_ptr).load();
        (*cell_ptr).store(*loaded_cell & !thr_bit);
    });
    #[cfg(feature = "atomics")]
    unsafe {
        loop {
            let next_cell = *loaded_cell & !thr_bit;
            match (*cell_ptr).compare_exchange_weak(
                *loaded_cell,
                next_cell,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    *loaded_cell = next_cell;
                    break;
                }
                Err(next_cell) => {
                    *loaded_cell = next_cell;
                }
            }
        }
    }
    unsafe { T::call(thr_idx, T::resume) };
}
