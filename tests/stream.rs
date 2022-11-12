#![cfg(not(loom))]
#![feature(sync_unsafe_cell)]
#![no_implicit_prelude]

use ::drone_core::{override_layout, stream};

override_layout! { r#"
[ram]
main = { origin = 0x20000000, size = "20K" }

[data]
ram = "main"

[stream]
ram = "main"

[stream.core0]
ram = "main"
size = "260"
init-primary = true

[stream.core1]
ram = "main"
size = "260"
"# }

stream! {
    layout => core0;
    /// Test doc attribute
    #[doc = "test attribute"]
    metadata => pub Stream0;
    /// Test doc attribute
    #[doc = "test attribute"]
    instance => pub STREAM0;
    global => true;
}

stream! {
    layout => core1;
    metadata => pub Stream1;
    instance => pub STREAM1;
}
