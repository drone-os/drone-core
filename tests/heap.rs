#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(extern_in_paths)]
#![feature(proc_macro_gen)]

extern crate drone_core;

use core::mem::size_of;
use drone_core::heap;
use std as core;

heap! {
  /// Test doc attribute
  #[doc = "test attribute"]
  pub struct Heap;

  size = 4096;
  pools = [
    [0x4; 512],
    [0x10; 128],
  ];
}

#[test]
fn size() {
  assert_eq!(size_of::<Heap>(), size_of::<heap::Pool>() * 2);
}
