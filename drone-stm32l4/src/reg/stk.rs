//! The processor has a 24-bit system timer, SysTick, that counts down from the
//! reload value to zero, reloads (wraps to) the value in the LOAD register on
//! the next clock edge, then counts down on subsequent clocks.

use drone_core::reg::{RawBits, RawValue};
use reg::Value;

const BASE: usize = 0xE000_E010;

define_reg! {
  name => Ctrl,
  desc => "SysTick control and status register.",
  addr => BASE + 0x00,
}

define_reg! {
  name => Load,
  desc => "SysTick reload value register.",
  addr => BASE + 0x04,
}

impl Value<Ctrl> {
  /// Sets `TICKINT` bit.
  ///
  /// SysTick exception request enable.
  pub fn tick(&mut self, enable: bool) -> &mut Value<Ctrl> {
    self.write(1, enable)
  }

  /// Sets `ENABLE` bit.
  ///
  /// Counter enable.
  pub fn enable(&mut self, enable: bool) -> &mut Value<Ctrl> {
    self.write(0, enable)
  }
}

impl Value<Load> {
  /// Specifies the start value to load into the VAL register when the counter
  /// is enabled and when it reaches 0.
  ///
  /// # Panics
  ///
  /// If `value` is greater than `0xFF_FFFF`.
  pub fn value(&mut self, value: u32) -> &mut Value<Load> {
    assert!(value <= 0xFF_FFFF);
    self.set(value)
  }
}
