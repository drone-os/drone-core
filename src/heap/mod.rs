//! Heap allocation.

mod pool;
mod allocator;

pub use self::allocator::Allocator;
pub use self::pool::{Pool, PoolFit};
pub use drone_macros::heap_imp;

/// Configure a heap allocator.
pub macro heap($($tokens:tt)*) {
  $crate::heap::heap_imp!($($tokens)*);
}
