use crate::{eprintln, platform};
use core::alloc::Layout;
use core::panic::PanicInfo;

#[panic_handler]
fn begin_panic(pi: &PanicInfo<'_>) -> ! {
    eprintln!("{}", pi);
    platform::reset()
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    eprintln!("memory allocation of {} bytes failed", layout.size());
    platform::reset()
}
