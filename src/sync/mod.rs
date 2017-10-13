//! Useful synchronization primitives.

pub mod mutex;
pub mod rwlock;
pub mod oneshot;

pub use self::mutex::{Mutex, MutexGuard};
pub use self::rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
