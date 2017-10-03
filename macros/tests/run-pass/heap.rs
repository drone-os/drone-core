#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(proc_macro)]
#![feature(slice_get_slice)]

extern crate alloc;
extern crate drone;
extern crate drone_macros;

use drone_macros::heap;
use std as core;

heap! {
  //! Test doc attribute
  #![doc = "test attribute"]
  size = 4096;
  pools = [
    [0x4; 512],
    [0x10; 128],
  ];
}

fn main() {}
