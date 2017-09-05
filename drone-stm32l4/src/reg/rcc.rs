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
  name => Cfgr,
  desc => "Clock configuration register.",
  addr => BASE + 0x08,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Pllcfgr => PllcfgrBits,
  desc => "PLL configuration register.",
  addr => BASE + 0x0C,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Cier => CierBits,
  desc => "Clock interrupt enable register.",
  addr => BASE + 0x18,
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

define_reg! {
  name => Apb1enr1 => Apb1enr1Bits,
  desc => "APB1 peripheral clock enable register 1.",
  addr => BASE + 0x58,
  alias => PERIPHERAL_ALIAS_BASE,
}

define_reg! {
  name => Bdcr => BdcrBits,
  desc => "Backup domain control register.",
  addr => BASE + 0x90,
  alias => PERIPHERAL_ALIAS_BASE,
}

/// Clock configuration register MSI clock range.
#[repr(u32)]
pub enum CrMsiRange {
  /// Range 0 around 100 kHz.
  Range100Khz = 0b0000,
  /// Range 1 around 200 kHz.
  Range200Khz = 0b0001,
  /// Range 2 around 400 kHz.
  Range400Khz = 0b0010,
  /// Range 3 around 800 kHz.
  Range800Khz = 0b0011,
  /// Range 4 around 1 MHz.
  Range1Mhz = 0b0100,
  /// Range 5 around 2 MHz.
  Range2Mhz = 0b0101,
  /// Range 6 around 4 MHz **(reset value)**.
  Range4Mhz = 0b0110,
  /// Range 7 around 8 MHz.
  Range8Mhz = 0b0111,
  /// Range 8 around 16 MHz.
  Range16Mhz = 0b1000,
  /// Range 9 around 24 MHz.
  Range24Mhz = 0b1001,
  /// Range 10 around 32 MHz.
  Range32Mhz = 0b1010,
  /// Range 11 around 48 MHz.
  Range48Mhz = 0b1011,
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

/// Backup domain control register rtc clock.
#[repr(u32)]
pub enum BdcrRtcClock {
  /// No clock.
  None = 0b00,
  /// LSE oscillator clock used as RTC clock.
  Lse = 0b01,
  /// LSI oscillator clock used as RTC clock.
  Lsi = 0b10,
  /// HSE oscillator clock divided by 32 used as RTC clock.
  Hse = 0b11,
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

  /// MSI clock PLL enable.
  fn msi_pll_enable(&mut self, enable: bool) -> &mut Self {
    self.write(2, enable)
  }

  /// MSI clock range selection.
  fn msi_range_selection(&mut self) -> &mut Self {
    self.write(3, true)
  }
}

/// PLL configuration register bits.
pub trait PllcfgrBits<T>: RawBits<Pllcfgr, T> {
  /// Main PLL PLLCLK output enable
  fn pllclk_enable(&mut self, enable: bool) -> &mut Self {
    self.write(24, enable)
  }
}

/// Clock interrupt enable register bits.
pub trait CierBits<T>: RawBits<Cier, T> {
  /// LSE clock security system interrupt enable.
  fn lse_css_interrupt_enable(&mut self, enable: bool) -> &mut Self {
    self.write(9, enable)
  }
}

/// Clock interrupt flag register bits.
pub trait CifrBits<T>: RawBits<Cifr, T> {
  /// LSE Clock security system interrupt flag.
  fn lse_css(&self) -> bool {
    self.read(9)
  }
}

/// Clock interrupt clear register bits.
pub trait CicrBits<T>: RawBits<Cicr, T> {
  /// LSE Clock security system interrupt clear.
  fn lse_css_clear(&mut self) -> &mut Self {
    self.write(9, true)
  }
}

/// AHB2 peripheral clock enable register bits.
pub trait Ahb2enrBits<T>: RawBits<Ahb2enr, T> {
  /// Enables an IO port clock.
  fn port_enable(&mut self, port: Ahb2enrIop, enable: bool) -> &mut Self {
    self.write(port as u32, enable)
  }
}

/// APB1 peripheral clock enable register 1 bits.
pub trait Apb1enr1Bits<T>: RawBits<Apb1enr1, T> {
  /// Power interface clock enable.
  fn power_enable(&mut self, enable: bool) -> &mut Self {
    self.write(28, enable)
  }
}

/// Backup domain control register bits.
pub trait BdcrBits<T>: RawBits<Bdcr, T> {
  /// LSE oscillator enable.
  fn lse_enable(&mut self, enable: bool) -> &mut Self {
    self.write(0, enable)
  }

  /// LSE oscillator ready.
  fn lse_ready(&self) -> bool {
    self.read(1)
  }

  /// LSE oscillator bypass.
  fn lse_bypass(&mut self, bypass: bool) -> &mut Self {
    self.write(2, bypass)
  }

  /// CSS on LSE enable.
  fn lse_css_enable(&mut self, enable: bool) -> &mut Self {
    self.write(5, enable)
  }

  /// CSS on LSE failure Detection.
  fn lse_css_failure(&self) -> bool {
    self.read(6)
  }
}

impl Value<Cr> {
  /// MSI clock ranges.
  pub fn msi_range(&mut self, range: CrMsiRange) -> &mut Value<Cr> {
    self.write_bits(range as u32, 4, 4)
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

  /// Main PLL division factor for PLLCLK (system clock).
  pub fn pllclk_factor(&mut self, factor: u32) -> &mut Self {
    assert!(factor == 2 || factor == 4 || factor == 6 || factor == 8);
    self.write_bits((factor >> 1) - 1, 2, 25)
  }

  /// Main PLL division factor for PLLSAI3CLK (SAI1 and SAI2 clock).
  pub fn pllsai3clk_factor(&mut self, factor: u32) -> &mut Self {
    assert!(factor == 7 || factor == 17);
    self.write_bits(if factor == 7 { 0b0 } else { 0b1 }, 1, 17)
  }

  /// Main PLL division factor for PLL48M1CLK (48 MHz clock).
  pub fn pll48m1clk_factor(&mut self, factor: u32) -> &mut Self {
    assert!(factor == 2 || factor == 4 || factor == 6 || factor == 8);
    self.write_bits((factor >> 1) - 1, 2, 21)
  }

  /// Main PLL multiplication factor for VCO.
  pub fn pll_output_factor(&mut self, factor: u32) -> &mut Self {
    assert!(factor >= 8);
    assert!(factor <= 86);
    self.write_bits(factor, 7, 8)
  }

  /// Division factor for the main PLL and audio PLL (PLLSAI1 and PLLSAI2) input
  /// clock.
  pub fn pll_input_factor(&mut self, factor: u32) -> &mut Self {
    assert!(factor >= 1);
    assert!(factor <= 8);
    self.write_bits(factor - 1, 3, 4)
  }
}

impl Value<Bdcr> {
  /// RTC clock source selection.
  pub fn rtc_source(&mut self, source: BdcrRtcClock) -> &mut Self {
    self.write_bits(source as u32, 2, 8)
  }
}
