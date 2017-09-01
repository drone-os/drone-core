//! Reset and clock control.

use core::mem::transmute;
use drone_core::reg::{RawBits, RawValue};
use reg::{Value, PERIPHERAL_ALIAS_BASE};

const BASE: usize = 0x4002_1000;

define_reg! {
  name => Cr => CrBits,
  desc => "Clock control register.",
  addr => BASE + 0x00,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Cfgr,
  desc => "Clock configuration register.",
  addr => BASE + 0x08,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Pllcfgr,
  desc => "PLL configuration register",
  addr => BASE + 0x0C,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Cifr => CifrBits,
  desc => "Clock interrupt flag register",
  addr => BASE + 0x1C,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Cicr => CicrBits,
  desc => "Clock interrupt clear register.",
  addr => BASE + 0x20,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Ahb2enr => Ahb2enrBits,
  desc => "AHB2 peripheral clock enable register.",
  addr => BASE + 0x4C,
  alias => PERIPHERAL_ALIAS_BASE,
}

/// AHB2 peripheral clock enable register port.
#[repr(u32)]
pub enum Ahb2enrIop {
  /// Port A.
  A = 0,
  /// Port B.
  B = 1,
  /// Port C.
  C = 2,
  /// Port D.
  D = 3,
  /// Port E.
  E = 4,
  /// Port F.
  F = 5,
  /// Port G.
  G = 6,
  /// Port H.
  H = 7,
  /// Port I.
  I = 8,
}

/// Clock configuration register PLL clock source.
#[repr(u32)]
pub enum PllcfgrPllSource {
  /// 00: No clock sent to PLL, PLLSAI1 and PLLSAI2
  None = 0b00,
  /// 01: MSI clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry
  Msi = 0b01,
  /// 10: HSI16 clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry
  Hsi16 = 0b10,
  /// 11: HSE clock selected as PLL, PLLSAI1 and PLLSAI2 clock entry
  Hse = 0b11,
}

/// Clock configuration register system clock.
#[repr(u32)]
pub enum CfgrSystemClock {
  /// MSI selected as system clock.
  Msi = 0b00,
  /// HSI16 selected as system clock.
  Hsi16 = 0b01,
  /// HSE selected as system clock.
  Hse = 0b10,
  /// PLL selected as system clock.
  Pll = 0b11,
}

/// Clock control register bits.
pub trait CrBits<T>: RawBits<Cr, T> {
  /// Main PLL enable.
  fn pll_enable(&mut self, enable: bool) -> &mut Self {
    self.write(24, enable)
  }

  /// HSE clock enable.
  fn hse_enable(&mut self, enable: bool) -> &mut Self {
    self.write(16, enable)
  }

  /// HSE crystal oscillator bypass.
  fn hse_bypass(&mut self, bypass: bool) -> &mut Self {
    self.write(18, bypass)
  }

  /// Main PLL clock ready flag.
  fn pll_ready(&self) -> bool {
    self.read(25)
  }

  /// HSE clock ready flag.
  fn hse_ready(&self) -> bool {
    self.read(17)
  }

  /// Clock security system enable.
  fn css_enable(&mut self, enable: bool) -> &mut Self {
    self.write(19, enable)
  }
}

/// Clock interrupt flag register bits.
pub trait CifrBits<T>: RawBits<Cifr, T> {
  /// Clock security system interrupt flag.
  fn css(&self) -> bool {
    self.read(8)
  }
}

/// Clock interrupt clear register bits.
pub trait CicrBits<T>: RawBits<Cicr, T> {
  /// Clock security system interrupt clear.
  fn css_clear(&mut self) -> &mut Self {
    self.write(8, true)
  }
}

/// AHB2 peripheral clock enable register bits.
pub trait Ahb2enrBits<T>: RawBits<Ahb2enr, T> {
  /// Enables an IO port clock.
  fn port_enable(&mut self, port: Ahb2enrIop, enable: bool) -> &mut Self {
    self.write(port as u32, enable)
  }
}

impl Value<Cfgr> {
  /// System clock switch.
  pub fn system_clock(&mut self, clock: CfgrSystemClock) -> &mut Value<Cfgr> {
    self.write_bits(clock as u32, 2, 0)
  }

  /// System clock switch status.
  pub fn system_clock_status(&mut self) -> CfgrSystemClock {
    unsafe { transmute(self.read_bits(2, 2)) }
  }
}

impl Value<Pllcfgr> {
  /// Main PLL, PLLSAI1 and PLLSAI2 entry clock source.
  pub fn pll_source(&mut self, source: PllcfgrPllSource) -> &mut Self {
    self.write_bits(source as u32, 2, 0)
  }
}
