#![feature(allocator_api)]

use crate::core::mem::size_of;
use std as core;

use drone_core::heap;

drone_core::config_override! { "
[memory.flash]
size = \"128K\"
origin = 0x08000000

[memory.ram]
size = \"20K\"
origin = 0x20000000

[heap]
size = \"10K\"
pools = [
    { block = \"4\", capacity = 896 },
    { block = \"32\", capacity = 80 },
    { block = \"256\", capacity = 16 },
]
" }

heap! {
    /// Test doc attribute
    #[doc = "test attribute"]
    pub struct Heap;
}

#[test]
fn size() {
    assert_eq!(size_of::<Heap>(), size_of::<heap::Pool>() * 3);
}
