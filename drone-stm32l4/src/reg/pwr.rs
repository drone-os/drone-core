//! Power registers.

use drone_core::reg::RawBits;
use reg::PERIPHERAL_ALIAS_BASE;

const BASE: usize = 0x4000_7000;

define_reg! {
  name => Cr1 => Cr1Bits,
  desc => "Power control register 1.",
  addr => BASE + 0x00,
  alias => PERIPHERAL_ALIAS_BASE,
}

/// Power control register 1 bits.
pub trait Cr1Bits<T>: RawBits<Cr1, T> {
  /// Disable backup domain write protection.
  fn backup_domain_protection_disable(&mut self, disable: bool) -> &mut Self {
    self.write(8, disable)
  }
}
