//! Dynamic memory allocation.
//!
//! Dynamic memory is crucial for Drone operation. Objectives like real-time
//! characteristics, high concurrency, small code size, fast execution have led
//! to Memory Pools design of the heap. All operations are lock-free and have
//! *O(1)* time complexity, which means they are deterministic.
//!
//! The continuous memory region for the heap is split into pools. A pool is
//! further split into fixed-sized blocks that hold actual allocations. A pool
//! is defined by its block-size and the number of blocks. The pools
//! configuration should be defined in the compile-time. A drawback of this
//! approach is that memory pools may need to be tuned for the application.
//!
//! # Usage
//!
//! Add the heap configuration to the `layout.toml`:
//!
//! ```toml
//! [heap]
//! size = "10K"
//! pools = [
//!     { block = "4", count = "896" },
//!     { block = "32", count = "80" },
//!     { block = "256", count = "16" },
//! ]
//! ```
//!
//! The `size` field should match the resulting size of the pools.
//!
//! Then in the application code:
//!
//! ```no_run
//! # #![feature(allocator_api)]
//! # #![feature(slice_ptr_get)]
//! # drone_core::override_layout! { r#"
//! # [ram]
//! # main = { origin = 0x20000000, size = "20K" }
//! # [data]
//! # ram = "main"
//! # [heap.main]
//! # ram = "main"
//! # size = "10K"
//! # pools = [
//! #     { block = "4", count = "896" },
//! #     { block = "32", count = "80" },
//! #     { block = "256", count = "16" },
//! # ]
//! # "# }
//! # fn main() {}
//! use drone_core::heap;
//!
//! // Define a concrete heap type with the layout defined in the layout.toml
//! heap! {
//!     // Heap name in `layout.toml`.
//!     layout => main;
//!     /// The main heap allocator generated from the `layout.toml`.
//!     metadata => pub Heap;
//!     /// The global allocator.
//!     #[global_allocator] // Use this heap as the global allocator.
//!     instance => pub HEAP;
//!     // Uncomment the following line to enable heap tracing feature:
//!     // enable_trace_stream => 31;
//! }
//! ```
//!
//! # Tuning
//!
//! Using empiric values for the memory pools layout may lead to undesired
//! memory fragmentation. Eventually the layout will need to be tuned for the
//! application. Drone can capture allocation statistics from the real target
//! device at the run-time and generate an optimized memory layout for this
//! specific application. Ideally this will result in zero fragmentation.
//!
//! The actual steps are platform-specific. Refer to the platform crate
//! documentation for instructions.

mod allocator;
mod pool;

pub use self::{
    allocator::{
        allocate, allocate_zeroed, binary_search, deallocate, grow, grow_zeroed, shrink, Allocator,
    },
    pool::Pool,
};
