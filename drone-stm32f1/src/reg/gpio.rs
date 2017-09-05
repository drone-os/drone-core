//! General-purpose and alternate-function I/Os.

use core::marker::PhantomData;
use drone::reg::{RawBits, RawValue};
use reg::{Value, PERIPHERAL_ALIAS_BASE};

const BASE: usize = 0x4001_0800;
const PORT_SIZE: usize = 0x0400;

/// Primitive types representing GPIO ports.
pub mod port {
  /// Port A.
  pub struct A;
  /// Port B.
  pub struct B;
  /// Port C.
  pub struct C;
  /// Port D.
  pub struct D;
  /// Port E.
  pub struct E;
  /// Port F.
  pub struct F;
  /// Port G.
  pub struct G;
}

/// Port `T` configuration register low.
pub struct Crl<T>(PhantomData<T>);

/// Port `T` configuration register high.
pub struct Crh<T>(PhantomData<T>);

/// Port `T` bit set/reset register.
pub struct Bsrr<T>(PhantomData<T>);

macro_rules! define_port_regs {
  ($name:ident, $i:expr) => {
    define_reg! {
      type => Crl<port::$name>,
      addr => BASE + PORT_SIZE * $i + 0x00,
      alias => PERIPHERAL_ALIAS_BASE,
    }

    define_reg! {
      type => Crh<port::$name>,
      addr => BASE + PORT_SIZE * $i + 0x04,
      alias => PERIPHERAL_ALIAS_BASE,
    }

    define_reg! {
      type => Bsrr<port::$name> => BsrrBits<port::$name>,
      addr => BASE + PORT_SIZE * $i + 0x10,
      alias => PERIPHERAL_ALIAS_BASE,
    }
  };
}

define_port_regs!(A, 0);
define_port_regs!(B, 1);
define_port_regs!(C, 2);
define_port_regs!(D, 3);
define_port_regs!(E, 4);
define_port_regs!(F, 5);
define_port_regs!(G, 6);

/// Port configuration register low pin.
#[repr(u32)]
pub enum CrlPin {
  /// Pin 0.
  P0 = 0x0,
  /// Pin 1.
  P1 = 0x4,
  /// Pin 2.
  P2 = 0x8,
  /// Pin 3.
  P3 = 0xC,
  /// Pin 4.
  P4 = 0x10,
  /// Pin 5.
  P5 = 0x14,
  /// Pin 6.
  P6 = 0x18,
  /// Pin 7.
  P7 = 0x1C,
}

/// Port configuration register high pin.
#[repr(u32)]
pub enum CrhPin {
  /// Pin 8.
  P8 = 0x0,
  /// Pin 9.
  P9 = 0x4,
  /// Pin 10.
  P10 = 0x8,
  /// Pin 11.
  P11 = 0xC,
  /// Pin 12.
  P12 = 0x10,
  /// Pin 13.
  P13 = 0x14,
  /// Pin 14.
  P14 = 0x18,
  /// Pin 15.
  P15 = 0x1C,
}

/// Port configuration register high mode.
#[repr(u32)]
pub enum CrMode {
  /// Analog input.
  InAnalog = 0b00_00,
  /// General purpose output push-pull, max speed 10 MHz.
  OutGpPuPu10 = 0b00_01,
  /// General purpose output push-pull, max speed 2 MHz.
  OutGpPuPu2 = 0b00_10,
  /// General purpose output push-pull, max speed 50 MHz.
  OutGpPuPu50 = 0b00_11,
  /// Floating input **(reset state)**.
  InFloating = 0b01_00,
  /// General purpose output Open-drain, max speed 10 MHz.
  OutGpOpDr10 = 0b01_01,
  /// General purpose output Open-drain, max speed 2 MHz.
  OutGpOpDr2 = 0b01_10,
  /// General purpose output Open-drain, max speed 50 MHz.
  OutGpOpDr50 = 0b01_11,
  /// Input with pull-up / pull-down.
  InPull = 0b10_00,
  /// Alternate function output Push-pull, max speed 10 MHz.
  OutAfPuPu10 = 0b10_01,
  /// Alternate function output Push-pull, max speed 2 MHz.
  OutAfPuPu2 = 0b10_10,
  /// Alternate function output Push-pull, max speed 50 MHz.
  OutAfPuPu50 = 0b10_11,
  /// Alternate function output Open-drain, max speed 10 MHz.
  OutAfOpDr10 = 0b11_01,
  /// Alternate function output Open-drain, max speed 2 MHz.
  OutAfOpDr2 = 0b11_10,
  /// Alternate function output Open-drain, max speed 50 MHz.
  OutAfOpDr50 = 0b11_11,
}

/// Port bit set/reset register pin.
#[repr(u32)]
pub enum BsrrPin {
  /// Pin 0.
  P0 = 0x0,
  /// Pin 1.
  P1 = 0x1,
  /// Pin 2.
  P2 = 0x2,
  /// Pin 3.
  P3 = 0x3,
  /// Pin 4.
  P4 = 0x4,
  /// Pin 5.
  P5 = 0x5,
  /// Pin 6.
  P6 = 0x6,
  /// Pin 7.
  P7 = 0x7,
  /// Pin 8.
  P8 = 0x8,
  /// Pin 9.
  P9 = 0x9,
  /// Pin 10.
  P10 = 0xa,
  /// Pin 11.
  P11 = 0xb,
  /// Pin 12.
  P12 = 0xc,
  /// Pin 13.
  P13 = 0xd,
  /// Pin 14.
  P14 = 0xe,
  /// Pin 15.
  P15 = 0xf,
}

/// Port bit set/reset register bits.
pub trait BsrrBits<T, U>: RawBits<Bsrr<T>, U> {
  /// Sets `pin` output.
  fn output(&mut self, pin: BsrrPin, enable: bool) -> &mut Self {
    self.write(
      if enable {
        pin as u32
      } else {
        pin as u32 + 0x10
      },
      true,
    )
  }
}

impl<T> Value<Crl<T>> {
  /// Sets pin mode configuration bits.
  pub fn pin_mode(
    &mut self,
    pin: CrlPin,
    config: CrMode,
  ) -> &mut Value<Crl<T>> {
    self.write_bits(config as u32, 4, pin as u32)
  }
}

impl<T> Value<Crh<T>> {
  /// Sets pin mode configuration bits.
  pub fn pin_mode(
    &mut self,
    pin: CrhPin,
    config: CrMode,
  ) -> &mut Value<Crh<T>> {
    self.write_bits(config as u32, 4, pin as u32)
  }
}
