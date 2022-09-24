//! Useful synchronization primitives.

pub mod linked_list;
pub mod new_spsc;
pub mod soft_atomic;
#[cfg(feature = "_atomics")]
pub mod spsc;

mod mutex;

pub use self::{
    linked_list::LinkedList,
    mutex::{Mutex, MutexGuard},
};
