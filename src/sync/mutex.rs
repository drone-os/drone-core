use crate::sync::linked_list::{LinkedList, Node};
use core::cell::UnsafeCell;
use core::fmt;
use core::future::Future;
use core::hint::unreachable_unchecked;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};

#[cfg(all(feature = "atomics", not(loom)))]
type DataLocked = core::sync::atomic::AtomicBool;
#[cfg(all(feature = "atomics", loom))]
type DataLocked = loom::sync::atomic::AtomicBool;
#[cfg(not(feature = "atomics"))]
type DataLocked = crate::sync::soft_atomic::Atomic<bool>;

#[cfg(all(feature = "atomics", not(loom)))]
type WaiterTaken = core::sync::atomic::AtomicBool;
#[cfg(all(feature = "atomics", loom))]
type WaiterTaken = loom::sync::atomic::AtomicBool;
#[cfg(not(feature = "atomics"))]
type WaiterTaken = crate::sync::soft_atomic::Atomic<bool>;

/// A mutual exclusion primitive useful for protecting shared data.
///
/// The mutex can be statically initialized or created via a [`new`]
/// constructor. Each mutex has a type parameter which represents the data that
/// it is protecting. The data can only be accessed through the RAII guards
/// returned from [`lock`] and [`try_lock`], which guarantees that the data is
/// only ever accessed when the mutex is locked.
///
/// [`new`]: Self::new
/// [`lock`]: Self::lock
/// [`try_lock`]: Self::try_lock
pub struct Mutex<T: ?Sized> {
    locked: DataLocked,
    waiters: LinkedList<Waiter>,
    data: UnsafeCell<T>,
}

/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] and [`DerefMut`] implementations.
///
/// This structure is created by the [`lock`] and [`try_lock`] methods on
/// [`Mutex`].
///
/// [`lock`]: Mutex::lock
/// [`try_lock`]: Mutex::try_lock
#[must_use = "if unused the Mutex will immediately unlock"]
pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
}

/// A future which resolves when the target mutex has been successfully
/// acquired.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct MutexLockFuture<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
    waiter: Option<NonNull<Node<Waiter>>>,
}

struct Waiter {
    taken: WaiterTaken,
    waker: UnsafeCell<MaybeUninit<Waker>>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for MutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Send for MutexLockFuture<'_, T> {}

impl<T> Mutex<T> {
    maybe_const_fn! {
        /// Creates a new mutex in an unlocked state ready for use.
        ///
        /// # Examples
        ///
        /// ```
        /// use drone_core::sync::Mutex;
        ///
        /// let mutex = Mutex::new(0);
        /// ```
        #[inline]
        pub const fn new(data: T) -> Self {
            Self {
                locked: DataLocked::new(false),
                waiters: LinkedList::new(),
                data: UnsafeCell::new(data),
            }
        }
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// assert_eq!(mutex.into_inner(), 0);
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Attempts to acquire this lock immediately.
    ///
    /// If the lock could not be acquired at this time, then [`None`] is
    /// returned. Otherwise, an RAII guard is returned. The lock will be
    /// unlocked when the guard is dropped.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if swap_atomic!(self.locked, true, Acquire) {
            None
        } else {
            Some(MutexGuard { mutex: self })
        }
    }

    /// Acquires this lock asynchronously.
    ///
    /// This method returns a future that will resolve once the lock has been
    /// successfully acquired.
    #[inline]
    pub fn lock(&self) -> MutexLockFuture<'_, T> {
        MutexLockFuture { mutex: self, waiter: None }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut() = 10;
    /// assert_eq!(*mutex.try_lock().unwrap(), 10);
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    fn unlock(&self) {
        let mut next_waker = None;
        unsafe {
            // This is the only place where nodes can be removed.
            self.waiters
                .drain_filter_raw(|waiter| (*waiter).is_taken())
                .for_each(|node| drop(Box::from_raw(node.cast_mut())));
            for waiter in self.waiters.iter_raw() {
                if let Some(waker) = (*waiter).take() {
                    next_waker = Some(waker);
                    break;
                }
            }
        }
        store_atomic!(self.locked, false, Release);
        if let Some(waker) = next_waker {
            waker.wake();
        }
    }
}

impl<'a, T: ?Sized> Future for MutexLockFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            if let Some(lock) = self.mutex.try_lock() {
                if let Some(waiter) = self.waiter.take() {
                    waiter.as_ref().take();
                }
                return Poll::Ready(lock);
            }
            if self.waiter.map_or(true, |waiter| waiter.as_ref().is_taken()) {
                let waiter = Box::into_raw(Box::new(Node::from(Waiter::from(cx.waker().clone()))));
                self.waiter = Some(NonNull::new_unchecked(waiter));
                self.mutex.waiters.push_raw(waiter);
                if let Some(lock) = self.mutex.try_lock() {
                    if let Some(waiter) = self.waiter.take() {
                        waiter.as_ref().take();
                    } else {
                        unreachable_unchecked();
                    }
                    return Poll::Ready(lock);
                }
            }
        }
        Poll::Pending
    }
}

impl<T: ?Sized> Drop for MutexLockFuture<'_, T> {
    fn drop(&mut self) {
        if let Some(waiter) = self.waiter {
            if unsafe { waiter.as_ref().take().is_none() } {
                // This future was awoken, but then dropped before it could acquire the lock.
                // Try to lock the mutex and then immediately unlock to wake up another thread.
                drop(self.mutex.try_lock());
            }
        }
    }
}

impl Waiter {
    fn take(&self) -> Option<Waker> {
        if swap_atomic!(self.taken, true, Acquire) {
            None
        } else {
            unsafe { Some((*self.waker.get()).assume_init_read()) }
        }
    }

    fn is_taken(&self) -> bool {
        load_atomic!(self.taken, Relaxed)
    }
}

impl From<Waker> for Waiter {
    fn from(waker: Waker) -> Self {
        Self { taken: WaiterTaken::new(false), waker: UnsafeCell::new(MaybeUninit::new(waker)) }
    }
}

impl Drop for Waiter {
    fn drop(&mut self) {
        if !load_atomic!(self.taken, Acquire) {
            unsafe { (*self.waker.get()).assume_init_read() };
        }
    }
}

impl<T> From<T> for Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use. This is
    /// equivalent to [`Mutex::new`].
    #[inline]
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<T: ?Sized + Default> Default for Mutex<T> {
    /// Creates a `Mutex<T>`, with the `Default` value for T.
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
    #[allow(clippy::option_if_let_else)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(guard) = self.try_lock() {
            f.debug_struct("Mutex").field("data", &&*guard).finish()
        } else {
            struct LockedPlaceholder;
            impl fmt::Debug for LockedPlaceholder {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("<locked>")
                }
            }
            f.debug_struct("Mutex").field("data", &LockedPlaceholder).finish()
        }
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MutexGuard").field("mutex", &self.mutex).finish()
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::sync::Arc;
    use core::future::Future;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    use futures::pin_mut;

    #[derive(Eq, PartialEq, Debug)]
    struct NonCopy(i32);

    struct Counter(AtomicUsize);

    impl Counter {
        fn to_waker(&'static self) -> Waker {
            unsafe fn clone(counter: *const ()) -> RawWaker {
                RawWaker::new(counter, &VTABLE)
            }
            unsafe fn wake(counter: *const ()) {
                unsafe { (*(counter as *const Counter)).0.fetch_add(1, Ordering::SeqCst) };
            }
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);
            unsafe { Waker::from_raw(RawWaker::new(self as *const _ as *const (), &VTABLE)) }
        }
    }

    #[test]
    fn try_lock() {
        let m = Mutex::new(());
        *m.try_lock().unwrap() = ();
    }

    #[test]
    fn lock() {
        static COUNTER: Counter = Counter(AtomicUsize::new(0));
        let waker = COUNTER.to_waker();
        let mut cx = Context::from_waker(&waker);
        let a = Arc::new(Mutex::new(1));
        let b = Arc::clone(&a);
        let c = Arc::clone(&b);
        let d = Arc::new(Mutex::new(0));
        let e = Arc::clone(&d);
        let f = async move {
            let mut b = b.lock().await;
            let mut _e = e.lock().await;
            *b *= 3;
        };
        let g = async move {
            let mut c = c.lock().await;
            *c *= 5;
        };
        pin_mut!(f);
        pin_mut!(g);
        let d = d.try_lock().unwrap();
        assert_eq!(*d, 0);
        assert_eq!(f.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(g.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(COUNTER.0.load(Ordering::SeqCst), 0);
        drop(d);
        assert_eq!(COUNTER.0.load(Ordering::SeqCst), 1);
        assert_eq!(g.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(f.as_mut().poll(&mut cx), Poll::Ready(()));
        assert_eq!(COUNTER.0.load(Ordering::SeqCst), 2);
        assert!(!a.waiters.is_empty());
        assert_eq!(g.as_mut().poll(&mut cx), Poll::Ready(()));
        assert!(a.waiters.is_empty());
        assert_eq!(*a.try_lock().unwrap(), 15);
    }

    #[test]
    fn into_inner() {
        let m = Mutex::new(NonCopy(10));
        assert_eq!(m.into_inner(), NonCopy(10));
    }

    #[test]
    fn into_inner_drop() {
        struct Foo(Arc<AtomicUsize>);
        impl Drop for Foo {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let num_drops = Arc::new(AtomicUsize::new(0));
        let m = Mutex::new(Foo(num_drops.clone()));
        assert_eq!(num_drops.load(Ordering::SeqCst), 0);
        {
            let _inner = m.into_inner();
            assert_eq!(num_drops.load(Ordering::SeqCst), 0);
        }
        assert_eq!(num_drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn get_mut() {
        let mut m = Mutex::new(NonCopy(10));
        *m.get_mut() = NonCopy(20);
        assert_eq!(m.into_inner(), NonCopy(20));
    }

    #[test]
    fn mutex_unsized() {
        let mutex: &Mutex<[i32]> = &Mutex::new([1, 2, 3]);
        {
            let b = &mut *mutex.try_lock().unwrap();
            b[0] = 4;
            b[2] = 5;
        }
        let comp: &[i32] = &[4, 2, 5];
        assert_eq!(&*mutex.try_lock().unwrap(), comp);
    }
}
