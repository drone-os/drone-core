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
//! There are multiple steps involved in heap usage:
//!
//! 1. Map a memory region in the `layout.ld`:
//!
//! ```ld
//! MEMORY
//! {
//!     /* ... */
//!     /* Continuous memory region for the heap: */
//!     HEAP (WX) : ORIGIN = 0x20000000, LENGTH = 256K
//!     /* ... */
//! }
//!
//! SECTIONS
//! {
//!     /* ... */
//!     /* Reserve a region for the heap and define HEAP_START symbol: */
//!     .heap : ALIGN(4)
//!     {
//!         HEAP_START = .;
//!         /* The number should match the LENGTH at the MEMORY section. */
//!         . += 0x40000;
//!     } > HEAP
//!     /* ... */
//! }
//! ```
//!
//! 2. Configure memory pools layout:
//!
//! ```
//! # #![feature(allocator_api)]
//! # fn main() {}
//! use drone_core::heap;
//!
//! heap! {
//!   /// The heap structure.
//!   pub struct Heap;
//!
//!   // The total size of the heap. Should match the layout.ld.
//!   size = 0x40000;
//!   // Declare the memory pools. The format is [BLOCK_SIZE; NUMBER_OF_BLOCKS]
//!   pools = [
//!     [0x4; 0x4000],
//!     [0x20; 0x800],
//!     [0x100; 0x100],
//!     [0x800; 0x20],
//!   ];
//! }
//! ```
//!
//! 3. Initialize the heap before the first use in the run-time.
//!
//! ```ignore
//! use drone_core::heap::Allocator;
//!
//! /// The global allocator.
//! #[global_allocator]
//! pub static mut HEAP: Heap = Heap::new();
//!
//! extern "C" {
//!     /// A value declared in the linker script at step 1.
//!     static mut HEAP_START: usize;
//! }
//!
//! // Your entry point.
//! fn main() {
//!     // ...
//!     unsafe {
//!         HEAP.init(&mut HEAP_START);
//!     }
//!     // ...
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

pub use self::{allocator::Allocator, pool::Pool};
