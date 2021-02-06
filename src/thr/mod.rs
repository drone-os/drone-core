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
//! thr::pool! {
//!     /// Thread pool storage.
//!     pool => pub ThrPool;
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

/// Defines a thread pool.
///
/// See [the module level documentation](self) for details.
#[doc(inline)]
pub use drone_core_macros::thr_pool as pool;

use crate::{
    fib::{Chain, RootFiber},
    token::Token,
};
use core::{
    cell::Cell,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Abstract thread pool.
pub trait ThreadPool: Sized + Sync + 'static {
    /// The thread type.
    type Thr: Thread;

    /// Returns a reference to the array of threads.
    ///
    /// A safe alternative to this function is calling [`ThrToken::to_thr`]
    /// method on an instance of a thread token.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't check for presence of the
    /// corresponding thread token.
    unsafe fn threads() -> &'static [Self::Thr];

    /// Returns a storage for the current thread index.
    fn current() -> &'static Current;
}

/// Abstract thread.
pub trait Thread: Sized + Sync + 'static {
    /// The thread pool type.
    type Pool: ThreadPool<Thr = Self>;

    /// The thread-local storage type.
    type Local: ThreadLocal;

    /// Returns a reference to the fiber chain.
    fn fib_chain(&self) -> &Chain;

    /// Returns a reference to the thread-local storage of the thread.
    ///
    /// A safe alternative to this method is calling [`local`] function when
    /// running on this thread.
    ///
    /// # Safety
    ///
    /// This method is unsafe because [`Thread`] is `Sync` while
    /// [`Thread::Local`] is not.
    unsafe fn local(&self) -> &Self::Local;
}

/// Abstract thread-local storage.
pub trait ThreadLocal: Sized + 'static {
    /// Returns a storage for the previous thread index.
    ///
    /// This method is safe because the type doesn't have public methods.
    fn preempted(&self) -> &Preempted;
}

/// Abstract token for a thread in a thread pool.
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

/// Current thread index.
pub struct Current(AtomicUsize);

/// Previous thread index.
pub struct Preempted(Cell<usize>);

impl Current {
    /// Creates a new `Current`.
    pub const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }
}

impl Preempted {
    /// Creates a new `Preempted`.
    pub const fn new() -> Self {
        Self(Cell::new(0))
    }
}

/// Returns a reference to the thread-local storage of the current thread.
///
/// The contents of this object can be customized with `thr::pool!` macro. See
/// [`the module-level documentation`](self) for details.
#[inline]
pub fn local<T: Thread>() -> &'static T::Local {
    unsafe { get_thr::<T>(T::Pool::current().0.load(Ordering::Relaxed)).local() }
}

/// Runs the fiber chain of the thread number `thr_hum`.
///
/// # Safety
///
/// The function is not reentrant.
#[inline(never)]
pub unsafe fn resume<T: Thread>(thr: &'static T) {
    unsafe { drop(thr.fib_chain().drain()) };
}

/// Runs the function `f` inside the thread number `thr_idx`.
///
/// # Safety
///
/// The function is not reentrant.
pub unsafe fn run<T: Thread>(thr_idx: usize, f: unsafe fn(&'static T)) {
    unsafe {
        let thr = get_thr::<T>(thr_idx);
        thr.local().preempted().0.set(T::Pool::current().0.load(Ordering::Relaxed));
        T::Pool::current().0.store(thr_idx, Ordering::Relaxed);
        f(thr);
        T::Pool::current().0.store(thr.local().preempted().0.get(), Ordering::Relaxed);
    }
}

unsafe fn get_thr<T: Thread>(thr_idx: usize) -> &'static T {
    unsafe { T::Pool::threads().get_unchecked(thr_idx) }
}
