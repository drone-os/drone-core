//! Utility module.

use drone::reg::{Delegate, ValuePointer};
use reg::{scb, Areg};

/// Wait for Interrupt.
pub fn wait_for_interrupt() {
  unsafe {
    asm!("wfi" :::: "volatile");
  }
}

/// Performs a system reset request.
pub fn reset_request() {
  let reg: Areg<scb::Aircr> = Areg::new();
  reg.ptr().modify(|reg| reg.unlock().reset());
}

/// Spins a specified amount of CPU cycles.
pub fn spin(mut _cycles: u32) {
  unsafe {
    asm!("
      0:
        subs $0, $0, #2
        bhi 0b
    " : "+r"(_cycles)
      :
      : "cc"
      : "volatile");
  }
}
