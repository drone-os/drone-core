use crate::{cpu, eprintln};
use core::{alloc::Layout, panic::PanicInfo};

#[panic_handler]
fn begin_panic(pi: &PanicInfo<'_>) -> ! {
    eprintln!("{}", pi);
    cpu::self_reset()
}

#[lang = "oom"]
fn oom(layout: Layout) -> ! {
    eprintln!("Couldn't allocate memory of size {}. Aborting!", layout.size());
    cpu::self_reset()
}
