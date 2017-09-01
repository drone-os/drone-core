//! Flash memory interface registers.

use drone_core::reg::{RawBits, RawValue};
use reg::{Value, PERIPHERAL_ALIAS_BASE};

const BASE: usize = 0x4002_2000;

define_reg! {
  name => Acr => AcrBits,
  desc => "Flash access control register.",
  addr => BASE + 0x00,
  alias => PERIPHERAL_ALIAS_BASE,
}

/// Flash access control register latency.
#[repr(u32)]
pub enum AcrWaitStates {
  /// Zero wait state
  Zero = 0b000,
  /// One wait state
  One = 0b001,
  /// Two wait states
  Two = 0b010,
  /// Three wait states
  Three = 0b011,
  /// Four wait states
  Four = 0b100,
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

  /// Flash half cycle access enable.
  fn half_cycle(&mut self, enable: bool) -> &mut Self {
    self.write(3, enable)
  }
}

impl Value<Acr> {
  /// The number of HCLK (AHB clock) period to the Flash access time.
  pub fn latency(&mut self, wait_states: AcrWaitStates) -> &mut Value<Acr> {
    self.write_bits(wait_states as u32, 3, 0)
  }
}
