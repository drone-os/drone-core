//! Flash memory interface registers.

use drone::reg::{RawBits, RawValue};
use reg::{Value, PERIPHERAL_ALIAS_BASE};

const BASE: usize = 0x4002_2000;

define_reg! {
  name => Acr => AcrBits,
  desc => "Flash access control register.",
  addr => BASE + 0x00,
  alias => PERIPHERAL_ALIAS_BASE,
}

/// Flash access control register bits.
pub trait AcrBits<T>: RawBits<Acr, T> {
  /// Prefetch enable.
  fn prefetch_enable(&mut self, enable: bool) -> &mut Self {
    self.write(8, enable)
  }

  /// Instruction cache enable.
  fn instruction_cache_enable(&mut self, enable: bool) -> &mut Self {
    self.write(9, enable)
  }

  /// Data cache enable.
  fn data_cache_enable(&mut self, enable: bool) -> &mut Self {
    self.write(10, enable)
  }
}

impl Value<Acr> {
  /// The number of HCLK (AHB clock) period to the Flash access time.
  pub fn latency(&mut self, wait_states: u32) -> &mut Value<Acr> {
    assert!(wait_states <= 4);
    self.write_bits(wait_states, 3, 0)
  }
}
