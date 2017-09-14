//! Useful synchronization primitives.

pub use spin::{Mutex, MutexGuard, Once, RwLock, RwLockReadGuard,
               RwLockWriteGuard};
