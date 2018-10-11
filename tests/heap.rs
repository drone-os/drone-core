#![feature(allocator_api)]
#![feature(const_fn)]

extern crate drone_core;

use core::alloc::Layout;
use core::mem::size_of;
use drone_core::heap;
use drone_core::heap::Pool;
use std as core;

heap! {
  /// Test doc attribute
  #[doc = "test attribute"]
  pub struct Heap;

  extern fn alloc_hook;
  extern fn dealloc_hook;

  size = 4096;
  pools = [
    [0x4; 512],
    [0x10; 128],
  ];
}

fn alloc_hook(_layout: Layout, _pool: &Pool) {}
fn dealloc_hook(_layout: Layout, _pool: &Pool) {}

#[test]
fn size() {
  assert_eq!(size_of::<Heap>(), size_of::<heap::Pool>() * 2);
}
