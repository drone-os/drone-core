#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate drone_core;

use drone_core::{
  reg::{self, marker::*, prelude::*},
  res,
};

reg! {
  pub mod RCC AHB2ENR;
  0 0x20 0 RReg WReg;
  GPIOAEN { 0 1 RRRegField WWRegField }
  GPIOBEN { 1 1 RRRegField WWRegField }
  GPIOCEN { 2 1 RRRegField WWRegField }
  GPIOARST { 3 1 RRRegField WWRegField }
  GPIOBRST { 4 1 RRRegField WWRegField }
}

reg! {
  pub mod GPIOA ODR;
  0 0x20 0 RReg WReg;
  ODR0 { 0 1 RRRegField WWRegField }
  ODR1 { 1 1 RRRegField WWRegField }
}

reg! {
  pub mod GPIOA IDR;
  0 0x20 0 RReg WReg;
  IDR0 { 0 1 RRRegField WWRegField }
  IDR1 { 1 1 RRRegField WWRegField }
}

reg! {
  pub mod GPIOB ODR;
  0 0x20 0 RReg WReg;
  ODR0 { 0 1 RRRegField WWRegField }
  ODR1 { 1 1 RRRegField WWRegField }
}

reg! {
  pub mod GPIOB IDR;
  0 0x20 0 RReg WReg;
  IDR0 { 0 1 RRRegField WWRegField }
}

reg! {
  pub mod GPIOC ODR;
  0 0x20 0 RReg WReg;
  ODR0 { 0 1 RRRegField WWRegField }
}

reg::index! {
  pub macro reg_idx;
  super;;

  pub mod RCC { AHB2ENR; }
  pub mod GPIOA { ODR; IDR; }
  pub mod GPIOB { ODR; IDR; }
  pub mod GPIOC { ODR; }
}

reg_idx! {
  pub struct RegIdx;
}

res! {
  pub trait Gpio {}

  RCC {
    AHB2ENR {
      0x20 RwReg Shared;
      GPIOEN { RwRwRegFieldBit }
      GPIORST { RwRwRegFieldBit Option }
    }
  }

  GPIO {
    ODR {
      0x20 RwReg;
      ODR0 { RwRwRegFieldBit }
      ODR1 { RwRwRegFieldBit Option }
    }
    IDR {
      0x20 RwReg Option;
      IDR0 { RwRwRegFieldBit }
      IDR1 { RwRwRegFieldBit Option }
    }
  }
}

res::map! {
  pub struct GpioA;

  impl Gpio for GpioA {}

  self;;

  RCC {
    AHB2ENR {
      AHB2ENR Shared;
      GPIOEN { GPIOAEN }
      GPIORST { GPIOARST Option }
    }
  }

  GPIO {
    GPIOA;
    ODR {
      ODR;
      ODR0 { ODR0 }
      ODR1 { ODR1 Option }
    }
    IDR {
      IDR Option;
      IDR0 { IDR0 }
      IDR1 { IDR1 Option }
    }
  }
}

res::map! {
  pub struct GpioB;

  impl Gpio for GpioB {}

  self;;

  RCC {
    AHB2ENR {
      AHB2ENR Shared;
      GPIOEN { GPIOBEN }
      GPIORST { GPIOBRST Option }
    }
  }

  GPIO {
    GPIOB;
    ODR {
      ODR;
      ODR0 { ODR0 }
      ODR1 { ODR1 Option }
    }
    IDR {
      IDR Option;
      IDR0 { IDR0 }
      IDR1 {}
    }
  }
}

res::map! {
  pub struct GpioC;

  impl Gpio for GpioC {}

  self;;

  RCC {
    AHB2ENR {
      AHB2ENR Shared;
      GPIOEN { GPIOCEN }
      GPIORST {}
    }
  }

  GPIO {
    GPIOC;
    ODR {
      ODR;
      ODR0 { ODR0 }
      ODR1 {}
    }
    IDR {}
  }
}

#[test]
fn res_macros() {
  #![allow(unused_variables)]
  let reg = unsafe { RegIdx::new() };
  let gpioa = res_gpio_a!(reg);
  let gpiob = res_gpio_b!(reg);
  let gpioc = res_gpio_c!(reg);
}

#[test]
fn concrete() {
  let reg = unsafe { RegIdx::new() };
  let gpio_c = res_gpio_c!(reg);
  let GpioRes {
    rcc_ahb2enr_gpioen,
    rcc_ahb2enr_gpiorst: (),
    gpio_odr,
    gpio_idr: (),
  } = gpio_c;
  let gpio_odr = gpio_odr.to_unsync();
  let gpio_odr = gpio_odr.to_sync();
  let SGpioOdrFields { odr0, odr1: () } = gpio_odr.into_fields();
  let odr0 = odr0.to_copy();
  let gpio_odr =
    CGpioOdr::from_fields(CGpioOdrFields::<GpioC> { odr0, odr1: () });
  let gpioc::Odr { odr0 } = gpio_odr;
  let gpio_odr = gpioc::Odr { odr0 };
  let rcc_ahb2enr_gpioen = rcc_ahb2enr_gpioen.to_copy();
  if false {
    gpio_odr.store(|r| r.set_odr0());
    gpio_odr.odr0.read_bit();
    rcc_ahb2enr_gpioen.read_bit();
  }
}

#[test]
fn generic_without_holes() {
  fn f<T: Gpio + GpioOdrOdr1 + GpioIdr + GpioIdrIdr1>(gpio: GpioRes<T>) {
    let GpioRes {
      rcc_ahb2enr_gpioen,
      rcc_ahb2enr_gpiorst: _,
      gpio_odr,
      gpio_idr: _,
    } = gpio;
    let gpio_odr = gpio_odr.to_unsync();
    let gpio_odr = gpio_odr.to_sync();
    let SGpioOdrFields { odr0, odr1 } = gpio_odr.into_fields();
    let odr0 = odr0.to_copy();
    let odr1 = odr1.to_copy();
    let gpio_odr = T::CGpioOdr::from_fields(CGpioOdrFields { odr0, odr1 });
    let rcc_ahb2enr_gpioen = rcc_ahb2enr_gpioen.to_copy();
    if false {
      let mut val = gpio_odr.load().val();
      gpio_odr.odr0().set(&mut val);
      gpio_odr.odr1().set(&mut val);
      gpio_odr.store_val(val);
      rcc_ahb2enr_gpioen.read_bit();
    }
  }
  let reg = unsafe { RegIdx::new() };
  let gpio_a = res_gpio_a!(reg);
  f(gpio_a);
}

#[test]
fn generic_with_holes() {
  fn f<T: Gpio>(gpio: GpioRes<T>) {
    let GpioRes {
      rcc_ahb2enr_gpioen,
      rcc_ahb2enr_gpiorst: _,
      gpio_odr,
      gpio_idr: _,
    } = gpio;
    let gpio_odr = gpio_odr.to_unsync();
    let gpio_odr = gpio_odr.to_sync();
    let SGpioOdrFields { odr0, odr1 } = gpio_odr.into_fields();
    let odr0 = odr0.to_sync();
    let gpio_odr = T::SGpioOdr::from_fields(SGpioOdrFields { odr0, odr1 });
    let rcc_ahb2enr_gpioen = rcc_ahb2enr_gpioen.to_copy();
    if false {
      let mut val = gpio_odr.load().val();
      gpio_odr.odr0().set(&mut val);
      gpio_odr.store_val(val);
      rcc_ahb2enr_gpioen.read_bit();
    }
  }
  let reg = unsafe { RegIdx::new() };
  let gpio_c = res_gpio_c!(reg);
  f(gpio_c);
}
