//! Collection types.
//!
//! Drone's collection library provides efficient implementations of the most
//! common lock-free data structures.

pub mod linked_list;

pub use self::linked_list::LinkedList;
