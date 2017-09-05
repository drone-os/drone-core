//! Reset and clock control.

use core::mem::transmute;
use drone::reg::{RawBits, RawValue};
use reg::{Value, PERIPHERAL_ALIAS_BASE};

const BASE: usize = 0x4002_1000;

define_reg! {
  name => Cr => CrBits,
  desc => "Clock control register.",
  addr => BASE + 0x00,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Cfgr => CfgrBits,
  desc => "Clock configuration register.",
  addr => BASE + 0x04,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Cir => CirBits,
  desc => "Clock interrupt register.",
  addr => BASE + 0x08,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Apb2enr => Apb2enrBits,
  desc => "APB2 peripheral clock enable register.",
  addr => BASE + 0x18,
  alias => PERIPHERAL_ALIAS_BASE,
}

/// APB2 peripheral clock enable register port.
#[repr(u32)]
pub enum Apb2enrIop {
  /// Port A.
  A = 2,
  /// Port B.
  B = 3,
  /// Port C.
  C = 4,
  /// Port D.
  D = 5,
  /// Port E.
  E = 6,
  /// Port F.
  F = 7,
  /// Port G.
  G = 8,
}

/// Clock configuration register PLL clock source.
pub enum CfgrPllSource {
  /// HSI oscillator clock / 2.
  Hsi,
  /// HSE oscillator clock.
  Hse,
}

/// Clock configuration register PLL HSE prescaler.
pub enum CfgrPllHsePrescaler {
  /// HSE clock not divided.
  None,
  /// HSE clock divided by 2.
  Div2,
}

/// Clock configuration register system clock.
#[repr(u32)]
pub enum CfgrSystemClock {
  /// HSI selected as system clock.
  Hsi = 0b00,
  /// HSE selected as system clock.
  Hse = 0b01,
  /// PLL selected as system clock.
  Pll = 0b10,
}

/// Clock control register bits.
pub trait CrBits<T>: RawBits<Cr, T> {
  /// PLL enable.
  fn pll_enable(&mut self, enable: bool) -> &mut Self {
    self.write(24, enable)
  }

  /// HSE clock enable.
  fn hse_enable(&mut self, enable: bool) -> &mut Self {
    self.write(16, enable)
  }

  /// External high-speed clock bypass.
  fn hse_bypass(&mut self, bypass: bool) -> &mut Self {
    self.write(18, bypass)
  }

  /// PLL clock ready flag.
  fn pll_ready(&self) -> bool {
    self.read(25)
  }

  /// External high-speed clock ready flag.
  fn hse_ready(&self) -> bool {
    self.read(17)
  }

  /// Clock security system enable.
  fn css_enable(&mut self, enable: bool) -> &mut Self {
    self.write(19, enable)
  }
}

/// Clock configuration register bits.
pub trait CfgrBits<T>: RawBits<Cfgr, T> {
  /// PLL entry clock source.
  fn pll_source(&mut self, source: CfgrPllSource) -> &mut Self {
    self.write(
      16,
      match source {
        CfgrPllSource::Hsi => false,
        CfgrPllSource::Hse => true,
      },
    )
  }

  /// HSE divider for PLL entry.
  fn pll_hse_prescaler(&mut self, prescaler: CfgrPllHsePrescaler) -> &mut Self {
    self.write(
      17,
      match prescaler {
        CfgrPllHsePrescaler::None => false,
        CfgrPllHsePrescaler::Div2 => true,
      },
    )
  }
}

/// Clock interrupt register bits.
pub trait CirBits<T>: RawBits<Cir, T> {
  /// Clock security system interrupt clear.
  fn css_clear(&mut self) -> &mut Self {
    self.write(23, true)
  }

  /// Clock security system interrupt flag.
  fn css(&self) -> bool {
    self.read(7)
  }
}

/// APB2 peripheral clock enable register bits.
pub trait Apb2enrBits<T>: RawBits<Apb2enr, T> {
  /// Enables an IO port clock.
  fn port_enable(&mut self, port: Apb2enrIop, enable: bool) -> &mut Self {
    self.write(port as u32, enable)
  }
}

impl Value<Cfgr> {
  /// PLL multiplication factor.
  ///
  /// # Panics
  ///
  /// If `value` is less than `0x2` or greater than `0x10`.
  pub fn pll_multiplication(&mut self, value: u32) -> &mut Value<Cfgr> {
    assert!(value >= 0x2);
    assert!(value <= 0x10);
    self.write_bits(value - 2, 4, 18)
  }

  /// System clock switch.
  pub fn system_clock(&mut self, clock: CfgrSystemClock) -> &mut Value<Cfgr> {
    self.write_bits(clock as u32, 2, 0)
  }

  /// System clock switch status.
  pub fn system_clock_status(&mut self) -> CfgrSystemClock {
    unsafe { transmute(self.read_bits(2, 2)) }
  }
}
