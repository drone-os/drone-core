//! The Threads module.
//!
//! Drone is a hard real-time operating system.  It uses interrupt-based
//! preemptive priority scheduling, where tasks with same priorities are
//! executed cooperatively. A task unit, called Fiber in Drone, is a stack-less
//! co-routine programmed with Rust async/await and/or generator syntax.
//!
//! A Drone application maps available prioritized interrupts to Drone threads.
//! The number of threads is always static but configurable. Any number of
//! fibers can be attached to particular threads, see [`fib`] for details. The
//! Drone application configures its own thread type, which implements
//! [`Thread`][`thr::Thread`], and creates a continuous array of this type.
//!
//! ```
//! # fn main() {}
//! use drone_core::thr;
//!
//! thr! {
//!     // Path to the array of threads.
//!     use THREADS;
//!
//!     /// The thread object.
//!     pub struct Thr {
//!         // You can add your own fields to the thread object. These fields will be
//!         // accessible through `to_thr` method of thread tokens. The types of
//!         // these fields should be `Sync`.
//!         pub foo: bool = false;
//!     }
//!
//!     // This is a part of `Thr` that can be accessed with `thr::local` function.
//!     /// The thread-local storage.
//!     pub struct ThrLocal {
//!         // You can add your own fields here with the same syntax as above.
//!         // Note that the initializer uses the special `index` variable, that
//!         // has the value of the position of the thread within the threads array.
//!         // The types of these fields shouldn't necessarily be `Sync`.
//!         pub bar: usize = index;
//!     }
//! }
//!
//! // This is for example only. Platform crates should provide macros to
//! // automatically generate this.
//! static mut THREADS: [Thr; 2] = [Thr::new(0), Thr::new(1)];
//! ```

pub mod prelude;

mod preempted;
mod task;

pub use self::{
    preempted::{local, PreemptedCell},
    task::TaskCell,
};

use self::preempted::preempt;
use crate::{
    fib::{Chain, FiberRoot},
    token::Token,
};

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
    /// Returns a reference to the task context cell.
    ///
    /// This method is safe because the type doesn't have public methods.
    fn task(&self) -> &TaskCell;

    /// Returns a reference to the previous thread index cell.
    ///
    /// This method is safe because the type doesn't have public methods.
    fn preempted(&self) -> &PreemptedCell;
}

/// The base trait for a thread token.
///
/// # Safety
///
/// [`ThrToken::THR_NUM`] must be a valid index in [`ThrToken::Thr`]'s array
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
    const THR_NUM: usize;

    /// Returns a reference to the thread object.
    #[inline]
    fn to_thr(self) -> &'static Self::Thr {
        unsafe { get_thr::<Self>() }
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

/// The thread handler function.
///
/// # Safety
///
/// The function is not reentrant.
pub unsafe fn thread_resume<T: ThrToken>() {
    let thr = get_thr::<T>();
    preempt(thr.local().preempted(), T::THR_NUM, || {
        thr.fib_chain().drain();
    })
}

unsafe fn get_thr<T: ThrToken>() -> &'static T::Thr {
    &*T::Thr::first().add(T::THR_NUM)
}
