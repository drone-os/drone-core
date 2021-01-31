//! Useful synchronization primitives.

pub mod linked_list;
pub mod spsc;

mod mutex;

pub use self::{
    linked_list::LinkedList,
    mutex::{Mutex, MutexGuard},
};
