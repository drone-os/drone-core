//! The System control block (SCB) provides system implementation information,
//! and system control. This includes configuration, control, and reporting of
//! the system exceptions.


use drone_core::reg::{RawBits, RawValue};
use reg::Value;


const BASE: usize = 0xE000_ED00;


define_reg! {
  name => Aircr,
  desc => "Application Interrupt and Reset Control Register.",
  addr => BASE + 0x0C,
}


impl Value<Aircr> {
  /// Register writes must include this method, otherwise the write is ignored.
  pub fn unlock(&mut self) -> &mut Value<Aircr> {
    let value = self.get();
    self.set(value ^ 0xFFFF_0000)
  }


  /// System Reset Request.
  pub fn reset(&mut self) -> &mut Value<Aircr> {
    self.write(2, true)
  }
}
