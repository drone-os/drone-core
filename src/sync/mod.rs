//! Useful synchronization primitives.

pub mod linked_list;
pub mod spsc;

mod mutex;
mod rwlock;

pub use self::{
    linked_list::LinkedList,
    mutex::{Mutex, MutexGuard},
    rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
