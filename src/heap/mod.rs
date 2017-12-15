//! Dynamic memory allocation.
//!
//! Drone provides an efficient lock-free allocator based on an array of
//! fixed-size memory pools. It allows deterministic constant time allocations
//! and deallocations well-suited for real-time systems. It doesn't need to
//! store allocation metadata or each allocation, neither an expensive run-time
//! initialization. The drawback is that it may need to be tuned for the
//! particular application.
//!
//! # Configuration
//!
//! Allocator is configured statically by `heap!` macro.
//!
//! ```
//! # #![feature(alloc)]
//! # #![feature(allocator_api)]
//! # #![feature(const_fn)]
//! # #![feature(decl_macro)]
//! # #![feature(slice_get_slice)]
//! # extern crate alloc;
//! # extern crate drone;
//! # fn main() {}
//! # use std as core;
//! use drone::heap;
//!
//! heap! {
//!   /// The allocator struct.
//!   Heap;
//!   /// The global allocator.
//!   // Uncomment the following line to use it as a global allocator.
//!   // #[global_allocator]
//!   ALLOC;
//!
//!   // The size of the heap should be known at the compile-time. It should
//!   // equal the sum of all defined pools.
//!   size = 0x40000;
//!   // Use 4 pools of different size.
//!   pools = [
//!     [0x4; 0x4000],  // 4-byte blocks with the capacity of 0x4000
//!     [0x20; 0x800],  // 32-byte blocks with the capacity of 0x800
//!     [0x100; 0x100], // 256-byte blocks with the capacity of 0x100
//!     [0x800; 0x20],  // 2048-byte blocks with the capacity of 0x20
//!   ];
//! }
//! ```
//!
//! # Initialization
//!
//! Allocator needs a simple run-time initialization before using.
//!
//! ```
//! # #![feature(alloc)]
//! # #![feature(allocator_api)]
//! # #![feature(const_fn)]
//! # #![feature(decl_macro)]
//! # #![feature(slice_get_slice)]
//! # extern crate alloc;
//! # extern crate drone;
//! # use std as core;
//! # pub mod symbols { #[no_mangle] pub static HEAP_START: usize = 0; }
//! use drone::heap;
//!
//! heap!(Heap; ALLOC);
//!
//! fn main() {
//!   extern "C" {
//!     // A symbol defined in the linker-script. Represents the beginning of
//!     // the heap region.
//!     static mut HEAP_START: usize;
//!   }
//!
//!   unsafe {
//!     ALLOC.init(&mut HEAP_START);
//!   }
//! }
//! ```
//!
//! # Operation
//!
//! Allocation steps:
//!
//! 1. Given a size of memory to allocate, binary search for a pool with the
//!    lower size of blocks which can fit the size. This step should compute in
//!    *O(log N)* time, where *N* is the total number of pools.
//!
//! 2. Get a free block from the pool. This step should compute in *O(1)* time.
//!    If the pool is exhausted, retry this step with the next bigger pool.
//!
//! Deallocation steps:
//!
//! 1. Given a pointer of the memory to deallocate, binary search for a pool for
//!    which the pointer belongs. This step should compute in *O(log N)* time,
//!    where *N* is the total number of pools.
//!
//! 2. Mark a block as free in the pool. This step should compute in *O(1)*
//!    time.

mod pool;
mod allocator;

pub use self::allocator::Allocator;
pub use self::pool::Pool;
