//! Heap allocator.

mod allocator;

pub use self::allocator::Allocator;

/// Configure global allocator.
#[macro_export]
macro_rules! alloc {
  () => {
    /// Global allocator.
    #[global_allocator]
    pub static ALLOC: $crate::alloc::Allocator = unsafe {
      $crate::alloc::Allocator::new()
    };
  };
}
