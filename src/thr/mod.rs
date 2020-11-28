//! The Threads module.
//!
//! **NOTE** A Drone platform crate may re-export this module with its own
//! additions under the same name, in which case it should be used instead.
//!
//! Drone is a hard real-time operating system.  It uses interrupt-based
//! preemptive priority scheduling, where tasks with same priorities are
//! executed cooperatively. A task unit, called Fiber in Drone, is a stack-less
//! co-routine programmed with Rust async/await and/or generator syntax.
//!
//! A Drone application maps available prioritized interrupts to Drone threads.
//! The number of threads is always static but configurable. Any number of
//! fibers can be attached to particular threads, see [`fib`](crate::fib) for
//! details. The Drone application configures its own thread type, which
//! implements [`Thread`], and creates a continuous array of this type.
//!
//! ```
//! # fn main() {}
//! use drone_core::thr;
//!
//! thr! {
//!     // Path to the array of threads.
//!     array => THREADS;
//!
//!     /// The thread object.
//!     thread => pub Thr {
//!         // You can add your own fields to the thread object. These fields will be
//!         // accessible through `to_thr` method of thread tokens. The types of
//!         // these fields should be `Sync`.
//!         pub foo: bool = false;
//!     };
//!
//!     // This is a part of `Thr` that can be accessed with `thr::local` function.
//!     /// The thread-local storage.
//!     local => pub ThrLocal {
//!         // You can add your own fields here with the same syntax as above.
//!         // Note that the initializer uses the special `index` variable, that
//!         // has the value of the position of the thread within the threads array.
//!         // The types of these fields shouldn't necessarily be `Sync`.
//!         pub bar: usize = index;
//!     };
//! }
//!
//! // This is for example only. Platform crates should provide macros to
//! // automatically generate this.
//! static mut THREADS: [Thr; 2] = [Thr::new(0), Thr::new(1)];
//! ```

pub mod prelude;

use crate::{
    fib::{Chain, FiberRoot},
    token::Token,
};
use core::cell::Cell;

static mut CURRENT: usize = 0;

/// Thread-local previous thread index cell.
pub struct PreemptedCell(Cell<usize>);

/// Generic thread.
pub trait Thread: Sized + Sync + 'static {
    /// The thread-local storage.
    type Local: ThreadLocal;

    /// Returns a pointer to the first thread in the thread array.
    fn first() -> *const Self;

    /// Returns a reference to the fiber chain.
    fn fib_chain(&self) -> &Chain;

    /// Returns a reference to the thread-local storage of the thread.
    ///
    /// [`local`] function should be used instead of this method.
    ///
    /// # Safety
    ///
    /// This method is unsafe because [`Thread`] is `Sync` while
    /// [`Thread::Local`] is not.
    unsafe fn local(&self) -> &Self::Local;
}

/// Generic thread-local storage.
pub trait ThreadLocal: Sized + 'static {
    /// Returns a reference to the previous thread index cell.
    ///
    /// This method is safe because the type doesn't have public methods.
    fn preempted(&self) -> &PreemptedCell;
}

/// The base trait for a thread token.
///
/// # Safety
///
/// [`ThrToken::THR_IDX`] must be a valid index in [`ThrToken::Thr`]'s array
/// returned by [`Thread::first`] method.
pub unsafe trait ThrToken
where
    Self: Sized + Clone + Copy,
    Self: Send + Sync + 'static,
    Self: Token,
{
    /// The thread type.
    type Thr: Thread;

    /// Position of the thread inside [`ThrToken::Thr`]'s array returned by
    /// [`Thread::first`] method.
    const THR_IDX: usize;

    /// Returns a reference to the thread object.
    #[inline]
    fn to_thr(self) -> &'static Self::Thr {
        unsafe { get_thr(Self::THR_IDX) }
    }

    /// Adds the fiber `fib` to the fiber chain.
    #[inline]
    fn add_fib<F: FiberRoot>(self, fib: F) {
        self.to_thr().fib_chain().add(fib);
    }

    /// Returns `true` if the fiber chain is empty.
    #[inline]
    fn is_empty(self) -> bool {
        self.to_thr().fib_chain().is_empty()
    }
}

impl PreemptedCell {
    /// Creates a new `PreemptedCell`.
    pub const fn new() -> Self {
        Self(Cell::new(0))
    }
}

/// Returns a reference to the thread-local storage of the current thread.
///
/// The contents of this object can be customized with `thr!` macro. See [`the
/// module-level documentation`](crate::thr) for details.
#[inline]
pub fn local<T: Thread>() -> &'static T::Local {
    unsafe { get_thr::<T>(CURRENT).local() }
}

/// Runs the fiber chain of the thread number `thr_hum`.
///
/// # Safety
///
/// The function is not reentrant.
pub unsafe fn thread_resume<T: Thread>(thr_idx: usize) {
    unsafe {
        thread_run::<T, _>(thr_idx, |thr| {
            thr.fib_chain().drain();
        });
    }
}

/// Runs the function `f` inside the thread number `thr_idx`.
///
/// # Safety
///
/// The function is not reentrant.
pub unsafe fn thread_call<T: Thread>(thr_idx: usize, f: unsafe extern "C" fn()) {
    unsafe {
        thread_run::<T, _>(thr_idx, |_| f());
    }
}

unsafe fn thread_run<T: Thread, F: FnOnce(&'static T)>(thr_idx: usize, f: F) {
    unsafe {
        let thr = get_thr::<T>(thr_idx);
        thr.local().preempted().0.set(CURRENT);
        CURRENT = thr_idx;
        f(thr);
        CURRENT = thr.local().preempted().0.get();
    }
}

unsafe fn get_thr<T: Thread>(thr_idx: usize) -> &'static T {
    unsafe { &*T::first().add(thr_idx) }
}
