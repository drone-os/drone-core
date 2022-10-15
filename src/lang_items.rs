use crate::{eprintln, platform};
use core::alloc::Layout;
use core::panic::PanicInfo;

#[panic_handler]
fn begin_panic(pi: &PanicInfo<'_>) -> ! {
    eprintln!("{}", pi);
    platform::reset()
}

#[lang = "oom"]
fn oom(layout: Layout) -> ! {
    eprintln!("Couldn't allocate memory of size {}. Aborting!", layout.size());
    platform::reset()
}
