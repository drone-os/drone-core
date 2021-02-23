//! The Threads module.
//!
//! **NOTE** A Drone platform crate may re-export this module with its own
//! additions under the same name, in which case it should be used instead.
//!
//! Drone is a hard real-time operating system. It uses interrupt-based
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
//!         pub bar: usize = index;
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
    soft::{SoftThrToken, SoftThread},
};

/// Defines a thread pool.
///
/// See [the module level documentation](self) for details.
#[doc(inline)]
pub use drone_core_macros::thr_pool as pool;

use crate::{
    fib::{Chain, RootFiber},
    token::Token,
};
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

/// Basic thread.
pub trait Thread: Sized + Sync + 'static {
    /// The thread-local storage type.
    type Local: Sized + 'static;

    /// Returns a reference to the array of opaque thread objects.
    ///
    /// To obtain a non-opaque reference to a thread object, use
    /// [`ThrToken::to_thr`] method on an instance of a thread token.
    fn threads() -> &'static [ThrOpaque<Self>];

    /// Returns a reference to the opaque current thread index storage.
    fn current() -> &'static AtomicOpaque<AtomicUsize>;

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
    #[inline]
    fn local() -> &'static Self::Local {
        let current = Self::current().0.load(Ordering::Relaxed);
        unsafe { Self::threads().get_unchecked(current).reveal().local_opaque().reveal() }
    }

    /// Resumes each fiber attached to the thread.
    ///
    /// # Safety
    ///
    /// The method is not reentrant.
    #[inline(never)]
    unsafe fn resume(&self) {
        unsafe { drop(self.fib_chain().drain()) };
    }

    /// Runs the function `f` inside the thread number `thr_idx`.
    ///
    /// # Safety
    ///
    /// * The function is not reentrant.
    /// * `thr_idx` must be a valid index within [`Thread::threads`] array.
    #[inline]
    unsafe fn call(thr_idx: usize, f: unsafe fn(&'static Self)) {
        unsafe {
            let preempted = Self::current().reveal().load(Ordering::Relaxed);
            let thr = Self::threads().get_unchecked(thr_idx).reveal();
            Self::current().reveal().store(thr_idx, Ordering::Relaxed);
            f(thr);
            Self::current().reveal().store(preempted, Ordering::Relaxed);
        }
    }
}

/// Token for a thread in a thread pool.
///
/// # Safety
///
/// * [`ThrToken::THR_IDX`] must be a valid index within [`ThreadPool::threads`]
///   array.
/// * At most one `ThrToken` type must exist for each thread.
pub unsafe trait ThrToken
where
    Self: Sized + Clone + Copy,
    Self: Send + Sync + 'static,
    Self: Token,
{
    /// The thread type.
    type Thread: Thread;

    /// Position of the thread within [`ThreadPool::threads`] array.
    const THR_IDX: usize;

    /// Returns a reference to the thread object.
    #[inline]
    fn to_thr(self) -> &'static Self::Thread {
        unsafe { Self::Thread::threads().get_unchecked(Self::THR_IDX).reveal() }
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

/// A wrapper type for incapsulating thread objects.
#[repr(transparent)]
pub struct ThrOpaque<T: Thread>(T);

/// A wrapper type for incapsulating thread-local storage objects.
#[repr(transparent)]
pub struct LocalOpaque<T: Thread>(T::Local);

/// A wrapper type for incapsulating various atomic variables for the threading
/// module.
#[repr(transparent)]
pub struct AtomicOpaque<T>(T);

unsafe impl<T: Thread> ::core::marker::Sync for LocalOpaque<T> {}

impl<T: Thread> LocalOpaque<T> {
    /// Creates a new `LocalOpaque`.
    #[inline]
    pub const fn new(local: T::Local) -> Self {
        Self(local)
    }

    // Safety: returned type is not `Sync`.
    unsafe fn reveal(&self) -> &T::Local {
        &self.0
    }
}

impl<T: Thread> ThrOpaque<T> {
    /// Creates a new `ThrOpaque`.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    fn reveal(&self) -> &T {
        &self.0
    }
}

impl<T> AtomicOpaque<T> {
    fn reveal(&self) -> &T {
        &self.0
    }
}

impl AtomicOpaque<AtomicUsize> {
    /// Creates a new zeroed `AtomicOpaque<AtomicUsize>`.
    #[inline]
    pub const fn default_usize() -> Self {
        Self(AtomicUsize::new(0))
    }
}

impl AtomicOpaque<AtomicU8> {
    /// Creates a new zeroed `AtomicOpaque<AtomicU8>`.
    #[inline]
    pub const fn default_u8() -> Self {
        Self(AtomicU8::new(0))
    }
}
