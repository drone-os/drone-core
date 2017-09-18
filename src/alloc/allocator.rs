use core_alloc::allocator::{Alloc, AllocErr, Layout};
use linked_list_allocator::LockedHeap;

/// Heap allocator.
pub struct Allocator {
  linked_list_allocator: LockedHeap,
}

impl Allocator {
  /// Creates an empty heap.
  ///
  /// # Safety
  ///
  /// Only one instance of the allocator must exists.
  #[inline]
  pub const unsafe fn new() -> Self {
    Self {
      linked_list_allocator: LockedHeap::empty(),
    }
  }

  /// Initializes the heap.
  ///
  /// # Safety
  ///
  /// Must be called exactly once and as early as possible.
  #[inline]
  pub unsafe fn init(&self, start: &mut u8, end: &u8) {
    let start = start as *mut _ as usize;
    let end = end as *const _ as usize;
    let count = end - start;
    self.linked_list_allocator.lock().init(start, count);
  }
}

unsafe impl<'a> Alloc for &'a Allocator {
  unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
    (&self.linked_list_allocator).alloc(layout)
  }

  unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
    (&self.linked_list_allocator).dealloc(ptr, layout)
  }
}
