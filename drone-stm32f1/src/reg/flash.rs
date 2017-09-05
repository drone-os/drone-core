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

/// Flash access control register latency.
#[repr(u32)]
pub enum AcrWaitStates {
  /// Zero wait state, if 0 < SYSCLK ≤ 24 MHz.
  Zero = 0b000,
  /// One wait state, if 24 MHz < SYSCLK ≤ 48 MHz.
  One = 0b001,
  /// Two wait states, if 48 MHz < SYSCLK ≤ 72 MHz.
  Two = 0b010,
}

/// Flash access control register bits.
pub trait AcrBits<T>: RawBits<Acr, T> {
  /// Prefetch buffer enable.
  fn prefetch_enable(&mut self, enable: bool) -> &mut Self {
    self.write(4, enable)
  }

  /// Flash half cycle access enable.
  fn half_cycle(&mut self, enable: bool) -> &mut Self {
    self.write(3, enable)
  }
}

impl Value<Acr> {
  /// The ratio of the SYSCLK (system clock) period to the Flash access time.
  pub fn latency(&mut self, wait_states: AcrWaitStates) -> &mut Value<Acr> {
    self.write_bits(wait_states as u32, 3, 0)
  }
}
