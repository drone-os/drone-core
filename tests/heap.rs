#![feature(allocator_api)]
#![feature(const_fn)]

use crate::core::mem::size_of;
use drone_core::heap::Pool;
use drone_core_macros::config_override;
use std as core;
use std::alloc::Layout;

use drone_core::heap;

config_override! { "
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

    use trace_alloc;
    use trace_dealloc;
    use trace_grow_in_place;
    use trace_shrink_in_place;
}

fn trace_alloc(_layout: Layout, _pool: &Pool) {}
fn trace_dealloc(_layout: Layout, _pool: &Pool) {}
fn trace_grow_in_place(_layout: Layout, _new_size: usize) {}
fn trace_shrink_in_place(_layout: Layout, _new_size: usize) {}

#[test]
fn size() {
    assert_eq!(size_of::<Heap>(), size_of::<heap::Pool>() * 3);
}
