#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![no_implicit_prelude]

use ::drone_core::{heap, override_layout};
use ::std::assert_eq;
use ::std::mem::size_of;

override_layout! { r#"
[ram]
main = { origin = 0x20000000, size = "20K" }

[data]
ram = "main"

[heap.primary]
ram = "main"
size = "10K"
pools = [
    { block = "4", count = "896" },
    { block = "32", count = "80" },
    { block = "256", count = "16" },
]

[heap.secondary]
ram = "main"
size = "6K"
pools = [
    { block = "4", count = "896" },
    { block = "32", count = "80" },
]
"# }

heap! {
    layout => primary;
    /// Test doc attribute
    #[doc = "test attribute"]
    metadata => pub HeapPrimary;
    /// Test doc attribute
    #[cfg_attr(not(feature = "std"), global_allocator)]
    #[doc = "test attribute"]
    instance => pub HEAP_PRIMARY;
}

heap! {
    layout => secondary;
    metadata => pub HeapSecondary;
    instance => pub HEAP_SECONDARY;
    enable_trace_stream => 5;
}

fn assert_global_alloc<T: ::core::alloc::GlobalAlloc>() {}

#[test]
fn size() {
    assert_global_alloc::<HeapPrimary>();
    assert_eq!(size_of::<HeapPrimary>(), size_of::<heap::Pool>() * 3);
    assert_eq!(size_of::<HeapSecondary>(), size_of::<heap::Pool>() * 2);
}
