//! General-purpose and alternate-function I/Os.

use core::marker::PhantomData;
use drone_core::reg::{RawBits, RawValue};
use reg::{Value, PERIPHERAL_ALIAS_BASE};

const BASE: usize = 0x4800_0000;
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
  /// Port H.
  pub struct H;
  /// Port I.
  pub struct I;
}

/// Port `T` mode register.
pub struct Moder<T>(PhantomData<T>);

/// Port `T` output type register.
pub struct Otyper<T>(PhantomData<T>);

/// Port `T` output speed register.
pub struct Ospeedr<T>(PhantomData<T>);

/// Port `T` pull-up/pull-down register.
pub struct Pupdr<T>(PhantomData<T>);

/// Port `T` bit set/reset register.
pub struct Bsrr<T>(PhantomData<T>);

macro_rules! define_port_regs {
  ($name:ident, $i:expr) => {
    define_reg! {
      type => Moder<port::$name>,
      addr => BASE + PORT_SIZE * $i + 0x00,
      alias => PERIPHERAL_ALIAS_BASE,
    }

    define_reg! {
      type => Otyper<port::$name>,
      addr => BASE + PORT_SIZE * $i + 0x04,
      alias => PERIPHERAL_ALIAS_BASE,
    }

    define_reg! {
      type => Ospeedr<port::$name>,
      addr => BASE + PORT_SIZE * $i + 0x08,
      alias => PERIPHERAL_ALIAS_BASE,
    }

    define_reg! {
      type => Pupdr<port::$name>,
      addr => BASE + PORT_SIZE * $i + 0x0C,
      alias => PERIPHERAL_ALIAS_BASE,
    }

    define_reg! {
      type => Bsrr<port::$name> => BsrrBits<port::$name>,
      addr => BASE + PORT_SIZE * $i + 0x18,
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
define_port_regs!(H, 7);
define_port_regs!(I, 8);

/// Port mode register pin.
#[repr(u32)]
pub enum ModerPin {
  /// Pin 0.
  P0 = 0x0,
  /// Pin 1.
  P1 = 0x2,
  /// Pin 2.
  P2 = 0x4,
  /// Pin 3.
  P3 = 0x6,
  /// Pin 4.
  P4 = 0x8,
  /// Pin 5.
  P5 = 0xA,
  /// Pin 6.
  P6 = 0xC,
  /// Pin 7.
  P7 = 0xE,
  /// Pin 8.
  P8 = 0x10,
  /// Pin 9.
  P9 = 0x12,
  /// Pin 10.
  P10 = 0x14,
  /// Pin 11.
  P11 = 0x16,
  /// Pin 12.
  P12 = 0x18,
  /// Pin 13.
  P13 = 0x1A,
  /// Pin 14.
  P14 = 0x1C,
  /// Pin 15.
  P15 = 0x1E,
}

/// Pin mode.
#[repr(u32)]
pub enum Mode {
  /// Input.
  Input = 0b00,
  /// General purpose output.
  Output = 0b01,
  /// Alternate function.
  Alternate = 0b10,
  /// Analog **(reset state)**.
  Analog = 0b11,
}

/// Port output type register pin.
#[repr(u32)]
pub enum OtyperPin {
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
  P10 = 0xA,
  /// Pin 11.
  P11 = 0xB,
  /// Pin 12.
  P12 = 0xC,
  /// Pin 13.
  P13 = 0xD,
  /// Pin 14.
  P14 = 0xE,
  /// Pin 15.
  P15 = 0xF,
}

/// Pin output type.
#[repr(u32)]
pub enum Otype {
  /// Push-pull **(reset state)**.
  PushPull = 0b0,
  /// Open-drain.
  OpenDrain = 0b1,
}

/// Port output speed register pin.
#[repr(u32)]
pub enum OspeedrPin {
  /// Pin 0.
  P0 = 0x0,
  /// Pin 1.
  P1 = 0x2,
  /// Pin 2.
  P2 = 0x4,
  /// Pin 3.
  P3 = 0x6,
  /// Pin 4.
  P4 = 0x8,
  /// Pin 5.
  P5 = 0xA,
  /// Pin 6.
  P6 = 0xC,
  /// Pin 7.
  P7 = 0xE,
  /// Pin 8.
  P8 = 0x10,
  /// Pin 9.
  P9 = 0x12,
  /// Pin 10.
  P10 = 0x14,
  /// Pin 11.
  P11 = 0x16,
  /// Pin 12.
  P12 = 0x18,
  /// Pin 13.
  P13 = 0x1A,
  /// Pin 14.
  P14 = 0x1C,
  /// Pin 15.
  P15 = 0x1E,
}

/// Pin output speed.
#[repr(u32)]
pub enum Ospeed {
  /// Low.
  Low = 0b00,
  /// Medium.
  Medium = 0b01,
  /// High.
  High = 0b10,
  /// Very high.
  VeryHigh = 0b11,
}

/// Port pull-up/pull-down register pin.
#[repr(u32)]
pub enum PupdrPin {
  /// Pin 0.
  P0 = 0x0,
  /// Pin 1.
  P1 = 0x2,
  /// Pin 2.
  P2 = 0x4,
  /// Pin 3.
  P3 = 0x6,
  /// Pin 4.
  P4 = 0x8,
  /// Pin 5.
  P5 = 0xA,
  /// Pin 6.
  P6 = 0xC,
  /// Pin 7.
  P7 = 0xE,
  /// Pin 8.
  P8 = 0x10,
  /// Pin 9.
  P9 = 0x12,
  /// Pin 10.
  P10 = 0x14,
  /// Pin 11.
  P11 = 0x16,
  /// Pin 12.
  P12 = 0x18,
  /// Pin 13.
  P13 = 0x1A,
  /// Pin 14.
  P14 = 0x1C,
  /// Pin 15.
  P15 = 0x1E,
}

/// Pin pull-up/pull-down configuration.
#[repr(u32)]
pub enum Pupd {
  /// No pull-up, pull-down.
  None = 0b00,
  /// Pull-up.
  PullUp = 0b01,
  /// Pull-down.
  PullDown = 0b10,
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

impl<T> Value<Moder<T>> {
  /// Sets pin mode configuration bits.
  pub fn pin_config(
    &mut self,
    pin: ModerPin,
    config: Mode,
  ) -> &mut Value<Moder<T>> {
    self.write_bits(config as u32, 2, pin as u32)
  }
}

impl<T> Value<Otyper<T>> {
  /// Sets pin output type configuration bits.
  pub fn pin_config(
    &mut self,
    pin: OtyperPin,
    config: Otype,
  ) -> &mut Value<Otyper<T>> {
    self.write_bits(config as u32, 1, pin as u32)
  }
}

impl<T> Value<Ospeedr<T>> {
  /// Sets pin output speed configuration bits.
  pub fn pin_config(
    &mut self,
    pin: OspeedrPin,
    config: Ospeed,
  ) -> &mut Value<Ospeedr<T>> {
    self.write_bits(config as u32, 2, pin as u32)
  }
}

impl<T> Value<Pupdr<T>> {
  /// Sets pin output speed configuration bits.
  pub fn pin_config(
    &mut self,
    pin: PupdrPin,
    config: Pupd,
  ) -> &mut Value<Pupdr<T>> {
    self.write_bits(config as u32, 2, pin as u32)
  }
}
