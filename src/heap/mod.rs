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
//! Add the heap configuration to the `Drone.toml`:
//!
//! ```toml
//! [heap]
//! size = "10K"
//! pools = [
//!     { block = "4", capacity = 896 },
//!     { block = "32", capacity = 80 },
//!     { block = "256", capacity = 16 },
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
//! # drone_core::config_override! { "
//! # [memory]
//! # flash = { size = \"128K\", origin = 0x08000000 }
//! # ram = { size = \"20K\", origin = 0x20000000 }
//! # [heap.main]
//! # size = \"10K\"
//! # pools = [
//! #     { block = \"4\", capacity = 896 },
//! #     { block = \"32\", capacity = 80 },
//! #     { block = \"256\", capacity = 16 },
//! # ]
//! # [linker]
//! # platform = \"arm\"
//! # " }
//! # fn main() {}
//! use drone_core::heap;
//!
//! // Define a concrete heap type with the layout defined in the Drone.toml
//! heap! {
//!     // Heap configuration key in `Drone.toml`.
//!     config => main;
//!     /// The main heap allocator generated from the `Drone.toml`.
//!     metadata => pub Heap;
//!     // Use this heap as the global allocator.
//!     global => true;
//!     // Uncomment the following line to enable heap tracing feature:
//!     // enable_trace_stream => 31;
//! }
//!
//! // Create a static instance of the heap type and declare it as the global
//! // allocator.
//! /// The global allocator.
//! #[global_allocator]
//! pub static HEAP: Heap = Heap::new();
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
