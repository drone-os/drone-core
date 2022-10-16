mod wake;

use self::wake::SoftWaker;
use crate::thr::{ThrExec, ThrToken, Thread};
use core::task::Waker;

/// Number of priority levels.
pub const PRIORITY_LEVELS: u8 = 27;

/// Returns the number of elements in [`SoftThread::pending`] array.
pub const fn pending_size<T: SoftThread>() -> usize {
    1 + row_size::<T>() * PRIORITY_LEVELS as usize
}

const fn row_size<T: SoftThread>() -> usize {
    (T::COUNT >> COL_BITS) as usize + (T::COUNT & (1 << COL_BITS) - 1 > 0) as usize
}

const fn cell_idx<T: SoftThread>(thr_idx: u16, priority: u8) -> usize {
    1 + row_size::<T>() * (PRIORITY_LEVELS - 1 - priority) as usize + (thr_idx >> COL_BITS) as usize
}

const fn pending_bit(thr_idx: u16) -> u32 {
    1 << (thr_idx & (1 << COL_BITS) - 1)
}

const COL_BITS: u32 = 5;

#[cfg(all(feature = "atomics", not(loom)))]
#[doc(hidden)]
pub type PendingState = core::sync::atomic::AtomicU32;
#[cfg(all(feature = "atomics", loom))]
#[doc(hidden)]
pub type PendingState = loom::sync::atomic::AtomicU32;
#[cfg(not(feature = "atomics"))]
#[doc(hidden)]
pub type PendingState = crate::sync::soft_atomic::Atomic<u32>;

#[cfg(all(feature = "atomics", not(loom)))]
#[doc(hidden)]
pub type PriorityState = core::sync::atomic::AtomicU8;
#[cfg(all(feature = "atomics", loom))]
#[doc(hidden)]
pub type PriorityState = loom::sync::atomic::AtomicU8;
#[cfg(not(feature = "atomics"))]
#[doc(hidden)]
pub type PriorityState = crate::sync::soft_atomic::Atomic<u8>;

/// Software-managed thread.
///
/// # Pending state structure
///
/// [`SoftThread::pending`] function returns a static array of `u32` with the
/// following structure:
///
/// `[H, C<L>, C<L>, ..., C<L-1>, C<L-1>, ..., C<0>, C<0>]`, where
///
/// * `H` - header
/// * `L` - maximum priority number
/// * `C<n>` - a set of pending status bits for each defined thread at the
///   priority level `n`
///
/// The header has the following bit structure:
///
/// `CCCCCPPP PPPPPPPP ...`, where
///
/// * `C` bits form a number of the currently running priority plus 1; value of
///   0 means no thread of this thread pool is currently running
/// * `P` - a set of pending status bits for each priority level
///
/// # Safety
///
/// [`SoftThread::pending`] must point to a static array with [`pending_size`]
/// number of elements.
pub unsafe trait SoftThread: Thread {
    /// Returns a raw pointer to the pending state storage.
    fn pending() -> *const PendingState;

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
    unsafe fn will_preempt(thr_idx: u16) -> bool {
        unsafe {
            let thr = Self::pool().add(usize::from(thr_idx));
            let priority = load_atomic!(*(*thr).priority(), Relaxed);
            set_pending(
                Self::pending(),
                cell_idx::<Self>(thr_idx, priority),
                pending_bit(thr_idx),
                priority,
            )
        }
    }

    /// Runs all pending threads with higher priorities than the current
    /// priority.
    fn preempt() {
        unsafe fn resume<T: Thread>(thr_idx: u16) {
            unsafe { T::call(thr_idx, T::resume) };
        }
        let pending = Self::pending();
        let row_size = row_size::<Self>();
        unsafe {
            if let Some((mut ptr, mut priority, prev_priority)) = row_start(pending, row_size) {
                loop {
                    row_run(&mut ptr, Self::COUNT, resume::<Self>);
                    if !row_next(pending, &mut ptr, &mut priority, prev_priority, row_size) {
                        break;
                    }
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
        unsafe {
            clear_pending(
                Self::SoftThread::pending(),
                cell_idx::<Self::SoftThread>(Self::THR_IDX, self.priority()),
                pending_bit(Self::THR_IDX),
            );
        }
    }

    /// Returns `true` if the thread is pending.
    #[inline]
    fn is_pending(self) -> bool {
        unsafe {
            is_pending(
                Self::SoftThread::pending(),
                cell_idx::<Self::SoftThread>(Self::THR_IDX, self.priority()),
                pending_bit(Self::THR_IDX),
            )
        }
    }

    /// Reads the priority of the thread.
    #[inline]
    fn priority(self) -> u8 {
        unsafe { load_atomic!(*self.to_soft_thr().priority(), Relaxed) }
    }

    /// Writes the priority of the thread.
    ///
    /// # Panics
    ///
    /// If `priority` is greater than or equals to [`PRIORITY_LEVELS`].
    #[inline]
    fn set_priority(self, priority: u8) {
        assert!(priority < PRIORITY_LEVELS);
        unsafe { store_atomic!(*self.to_soft_thr().priority(), priority, Relaxed) };
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

unsafe fn row_start(
    header: *const PendingState,
    row_size: usize,
) -> Option<(*const PendingState, u8, u8)> {
    #[cfg_attr(any(feature = "atomics", loom), allow(unused_assignments))]
    let (mut priority, mut prev_priority) = (0, 0);
    load_try_modify_atomic!(unsafe { &*header }, Relaxed, Acquire, |cursor| {
        priority = PRIORITY_LEVELS;
        prev_priority = (cursor >> priority) as u8;
        cursor_find_priority(cursor, &mut priority, prev_priority)
    })
    .ok()?;
    let ptr = unsafe { header.add(1 + row_size * usize::from(PRIORITY_LEVELS - priority)) };
    Some((ptr, priority, prev_priority))
}

unsafe fn row_run(ptr: &mut *const PendingState, thr_count: u16, resume: unsafe fn(u16)) {
    let mut thr_idx = 0;
    loop {
        let mut cell = load_atomic!(unsafe { &**ptr }, Relaxed);
        if cell == 0 {
            thr_idx += 1 << COL_BITS;
            if thr_idx >= thr_count {
                break;
            }
        } else {
            let mut pending_bit = 1;
            loop {
                if cell & pending_bit != 0 {
                    cell = modify_atomic!(unsafe { &**ptr }, Relaxed, Acquire, |cell| {
                        cell & !pending_bit
                    });
                    unsafe { resume(thr_idx) };
                }
                thr_idx += 1;
                if thr_idx == thr_count {
                    return;
                }
                pending_bit <<= 1;
                if pending_bit == 0 {
                    break;
                }
            }
        }
        *ptr = unsafe { ptr.add(1) };
    }
}

unsafe fn row_next(
    header: *const PendingState,
    ptr: &mut *const PendingState,
    priority: &mut u8,
    prev_priority: u8,
    row_size: usize,
) -> bool {
    #[cfg_attr(any(feature = "atomics", loom), allow(unused_assignments))]
    let mut next_priority = 0;
    load_modify_atomic!(unsafe { &*header }, Relaxed, Release, |cursor| {
        next_priority = *priority - 1;
        cursor_find_priority(cursor, &mut next_priority, prev_priority)
            .unwrap_or_else(|| cursor_set_priority(cursor, prev_priority))
    });
    if next_priority == prev_priority {
        return false;
    }
    if next_priority != *priority {
        *ptr = unsafe { ptr.add(row_size * usize::from(*priority - next_priority)) };
        *priority = next_priority;
    }
    true
}

unsafe fn set_pending(
    pending: *const PendingState,
    cell_idx: usize,
    pending_bit: u32,
    priority: u8,
) -> bool {
    fetch_or_atomic!(unsafe { &*pending.add(cell_idx) }, pending_bit, Release) & pending_bit == 0
        && fetch_or_atomic!(unsafe { &*pending }, 1 << priority, Release) >> PRIORITY_LEVELS
            < u32::from(priority + 1)
}

unsafe fn clear_pending(pending: *const PendingState, cell_idx: usize, pending_bit: u32) {
    fetch_and_atomic!(unsafe { &*pending.add(cell_idx) }, !pending_bit, Release);
}

unsafe fn is_pending(pending: *const PendingState, cell_idx: usize, pending_bit: u32) -> bool {
    load_atomic!(unsafe { &*pending.add(cell_idx) }, Relaxed) & pending_bit != 0
}

#[allow(clippy::inline_always)] // because it can be inside a critical section
#[inline(always)]
fn cursor_find_priority(cursor: u32, priority: &mut u8, prev_priority: u8) -> Option<u32> {
    loop {
        if *priority == prev_priority {
            break None;
        }
        let priority_bit = 1 << *priority - 1;
        if cursor & priority_bit != 0 {
            break Some(cursor_set_priority(cursor, *priority) & !priority_bit);
        }
        *priority -= 1;
    }
}

#[allow(clippy::inline_always)] // because it can be inside a critical section
#[inline(always)]
fn cursor_set_priority(cursor: u32, priority: u8) -> u32 {
    cursor & !(u32::MAX << PRIORITY_LEVELS) | u32::from(priority) << PRIORITY_LEVELS
}
