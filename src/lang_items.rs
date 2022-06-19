use crate::eprintln;
use core::{alloc::Layout, panic::PanicInfo};

extern "C" {
    fn drone_self_reset() -> !;
}

#[panic_handler]
fn begin_panic(pi: &PanicInfo<'_>) -> ! {
    eprintln!("{}", pi);
    abort()
}

#[lang = "oom"]
fn oom(layout: Layout) -> ! {
    eprintln!("Couldn't allocate memory of size {}. Aborting!", layout.size());
    abort()
}

fn abort() -> ! {
    unsafe { drone_self_reset() }
}
