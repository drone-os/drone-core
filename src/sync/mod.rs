//! Useful synchronization primitives.

pub mod spsc;

mod mutex;
mod rwlock;

pub use self::mutex::{Mutex, MutexGuard};
pub use self::rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
