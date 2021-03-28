//! The Threads module.
//!
//! **NOTE** A Drone platform crate may re-export this module with its own
//! additions under the same name, in which case it should be used instead.
//!
//! Drone is a hard real-time operating system. It uses preemptive priority
//! scheduling, where tasks with same priorities are executed cooperatively. A
//! task unit, called Fiber in Drone, is a stack-less co-routine programmed with
//! Rust async/await and/or generator syntax.
//!
//! The number of threads is always static but configurable. Any number of
//! fibers can be attached to a particular thread, see [`fib`](crate::fib) for
//! details. The Drone application configures its own thread type, which
//! implements [`Thread`], and creates a continuous array of this type.
//!
//! ```
//! # fn main() {}
//! use drone_core::thr;
//!
//! thr::pool! {
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
//!         pub bar: u16 = index;
//!     };
//!
//!     /// Thread token set.
//!     index => pub Thrs;
//!
//!     // Thread definitions.
//!     threads => {
//!         /// Example thread 1.
//!         pub thread1;
//!         /// Example thread 2.
//!         pub thread2;
//!     };
//! }
//! ```

pub mod prelude;

mod exec;
mod soft;

pub use self::{
    exec::{ExecOutput, ThrExec},
    soft::{pending_size, SoftThrToken, SoftThread, PRIORITY_LEVELS},
};

/// Defines a thread pool.
///
/// See [the module level documentation](self) for details.
#[doc(inline)]
pub use drone_core_macros::thr_pool as pool;

/// Defines a software-managed thread pool.
///
/// See [the module level documentation](self) for details.
#[doc(inline)]
pub use drone_core_macros::thr_soft as soft;

use crate::{
    fib::{Chain, RootFiber},
    token::Token,
};
use core::sync::atomic::{AtomicU16, Ordering};

/// Basic thread.
///
/// # Safety
///
/// * [`Thread::pool`] must point to an array with [`Thread::COUNT`] number of
///   elements.
/// * [`Thread::current`] value must be zero-initialized.
pub unsafe trait Thread: Sized + Sync + 'static {
    /// The thread-local storage type.
    type Local: Sized + 'static;

    /// Number of threads in the pool.
    const COUNT: u16;

    /// Returns a raw pointer to the thread pool.
    ///
    /// To obtain a safe reference to a thread object, use [`ThrToken::to_thr`]
    /// method on the corresponding thread token instance.
    fn pool() -> *const Self;

    /// Returns a raw pointer to the current thread index storage.
    fn current() -> *const AtomicU16;

    /// Returns a reference to the fiber chain.
    fn fib_chain(&self) -> &Chain;

    /// Returns a reference to the opaque thread-local storage.
    ///
    /// Non-opaque thread-local storage can be obtained through
    /// [`Thread::local`] function.
    fn local_opaque(&self) -> &LocalOpaque<Self>;

    /// Returns a reference to the thread-local storage for the current thread.
    ///
    /// The contents of this object can be customized with `thr::pool!`
    /// macro. See [`the module-level documentation`](self) for details.
    ///
    /// # Panics
    ///
    /// This function will panic if called outside of this thread pool.
    #[inline]
    fn local() -> &'static Self::Local {
        Self::local_checked().expect("getting thread-local outside of thread pool")
    }

    /// Returns a reference to the thread-local storage for the current thread.
    ///
    /// If called outside of this thread pool, returns `None`.
    ///
    /// The contents of this object can be customized with `thr::pool!`
    /// macro. See [`the module-level documentation`](self) for details.
    #[inline]
    fn local_checked() -> Option<&'static Self::Local> {
        unsafe {
            let current = (*Self::current()).load(Ordering::Relaxed);
            if current == 0 {
                None
            } else {
                Some((*Self::pool().add(usize::from(current) - 1)).local_opaque().reveal())
            }
        }
    }

    /// Resumes each fiber attached to the thread.
    ///
    /// # Safety
    ///
    /// The method is not reentrant.
    #[inline]
    unsafe fn resume(&self) {
        unsafe { drop(self.fib_chain().drain()) };
    }

    /// Runs the function `f` inside the thread number `thr_idx`.
    ///
    /// # Safety
    ///
    /// * The function is not reentrant.
    /// * `thr_idx` must be less than [`Thread::COUNT`].
    #[inline]
    unsafe fn call(thr_idx: u16, f: unsafe fn(&'static Self)) {
        unsafe {
            let preempted = (*Self::current()).load(Ordering::Relaxed);
            (*Self::current()).store(thr_idx + 1, Ordering::Relaxed);
            f(&*Self::pool().add(usize::from(thr_idx)));
            (*Self::current()).store(preempted, Ordering::Relaxed);
        }
    }
}

/// Token for a thread in a thread pool.
///
/// # Safety
///
/// * At most one trait implementation per thread must exist.
/// * [`ThrToken::THR_IDX`] must be less than [`ThrToken::Thread::COUNT`].
pub unsafe trait ThrToken
where
    Self: Sized + Clone + Copy,
    Self: Send + Sync + 'static,
    Self: Token,
{
    /// The thread type.
    type Thread: Thread;

    /// Position of the thread within [`Self::Thread::pool`] array.
    const THR_IDX: u16;

    /// Returns a reference to the thread object.
    #[inline]
    fn to_thr(self) -> &'static Self::Thread {
        unsafe { &*Self::Thread::pool().add(usize::from(Self::THR_IDX)) }
    }

    /// Adds the fiber `fib` to the fiber chain.
    #[inline]
    fn add_fib<F>(self, fib: F)
    where
        F: RootFiber + Send,
    {
        self.to_thr().fib_chain().add(fib);
    }

    /// Adds the fiber returned by `factory` to the fiber chain.
    ///
    /// This method is useful for non-`Send` fibers.
    #[inline]
    fn add_fib_factory<C, F>(self, factory: C)
    where
        C: FnOnce() -> F + Send + 'static,
        F: RootFiber,
    {
        self.to_thr().fib_chain().add(factory());
    }

    /// Returns `true` if the fiber chain is empty.
    #[inline]
    fn is_empty(self) -> bool {
        self.to_thr().fib_chain().is_empty()
    }
}

/// Thread-local storage wrapper for thread `T`.
///
/// The wrapper is always `Sync`, while `T::Local` is not necessarily
/// `Sync`. The contents of the wrapper can be revealed only by
/// [`Thread::local`] function, which guarantees that the contents doesn't leave
/// its thread.
#[repr(transparent)]
pub struct LocalOpaque<T: Thread>(T::Local);

unsafe impl<T: Thread> ::core::marker::Sync for LocalOpaque<T> {}

impl<T: Thread> LocalOpaque<T> {
    /// Creates a new `LocalOpaque`.
    #[inline]
    pub const fn new(local: T::Local) -> Self {
        Self(local)
    }

    unsafe fn reveal(&self) -> &T::Local {
        &self.0
    }
}
