#![feature(proc_macro_hygiene)]

use drone_core::{
    periph,
    reg::{marker::*, prelude::*},
    token::Token,
};

use drone_core::reg;

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

reg::unsafe_tokens! {
    pub macro unsafe_reg_tokens;
    super;;

    pub mod RCC { AHB2ENR; }
    pub mod GPIOA { ODR; IDR; }
    pub mod GPIOB { ODR; IDR; }
    pub mod GPIOC { ODR; }
}

unsafe_reg_tokens! {
    pub struct Regs;
}

periph! {
    pub trait GpioMap {}

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

periph::map! {
    pub macro periph_gpio_a;
    pub struct GpioA;
    impl GpioMap for GpioA {}
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

periph::map! {
    pub macro periph_gpio_b;
    pub struct GpioB;
    impl GpioMap for GpioB {}
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

periph::map! {
    pub macro periph_gpio_c;
    pub struct GpioC;
    impl GpioMap for GpioC {}
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
fn periph_macros() {
    #![allow(unused_variables)]
    let reg = unsafe { Regs::take() };
    let gpioa = periph_gpio_a!(reg);
    let gpiob = periph_gpio_b!(reg);
    let gpioc = periph_gpio_c!(reg);
}

#[test]
fn concrete() {
    let reg = unsafe { Regs::take() };
    let gpio_c = periph_gpio_c!(reg);
    let GpioPeriph {
        rcc_ahb2enr_gpioen,
        rcc_ahb2enr_gpiorst: (),
        gpio_odr,
        gpio_idr: (),
    } = gpio_c;
    let gpio_odr = gpio_odr.into_unsync();
    let gpio_odr = gpio_odr.into_sync();
    let SGpioOdrFields { odr0, odr1: () } = gpio_odr.into_fields();
    let odr0 = odr0.into_copy();
    let gpio_odr = CGpioOdr::from_fields(CGpioOdrFields::<GpioC> { odr0, odr1: () });
    let gpioc::Odr { odr0 } = gpio_odr;
    let gpio_odr = gpioc::Odr { odr0 };
    let rcc_ahb2enr_gpioen = rcc_ahb2enr_gpioen.into_copy();
    if false {
        gpio_odr.store(|r| r.set_odr0());
        gpio_odr.odr0.read_bit();
        rcc_ahb2enr_gpioen.read_bit();
    }
}

#[test]
fn generic_without_holes() {
    fn f<T: GpioMap + GpioOdrOdr1 + GpioIdr + GpioIdrIdr1>(gpio: GpioPeriph<T>) {
        let GpioPeriph {
            rcc_ahb2enr_gpioen,
            rcc_ahb2enr_gpiorst: _,
            gpio_odr,
            gpio_idr: _,
        } = gpio;
        let gpio_odr = gpio_odr.into_unsync();
        let gpio_odr = gpio_odr.into_sync();
        let SGpioOdrFields { odr0, odr1 } = gpio_odr.into_fields();
        let odr0 = odr0.into_copy();
        let odr1 = odr1.into_copy();
        let gpio_odr = T::CGpioOdr::from_fields(CGpioOdrFields { odr0, odr1 });
        let rcc_ahb2enr_gpioen = rcc_ahb2enr_gpioen.into_copy();
        if false {
            let mut val = gpio_odr.load().val();
            gpio_odr.odr0().set(&mut val);
            gpio_odr.odr1().set(&mut val);
            gpio_odr.store_val(val);
            rcc_ahb2enr_gpioen.read_bit();
        }
    }
    let reg = unsafe { Regs::take() };
    let gpio_a = periph_gpio_a!(reg);
    f(gpio_a);
}

#[test]
fn generic_with_holes() {
    fn f<T: GpioMap>(gpio: GpioPeriph<T>) {
        let GpioPeriph {
            rcc_ahb2enr_gpioen,
            rcc_ahb2enr_gpiorst: _,
            gpio_odr,
            gpio_idr: _,
        } = gpio;
        let gpio_odr = gpio_odr.into_unsync();
        let gpio_odr = gpio_odr.into_sync();
        let SGpioOdrFields { odr0, odr1 } = gpio_odr.into_fields();
        let odr0 = odr0.into_sync();
        let gpio_odr = T::SGpioOdr::from_fields(SGpioOdrFields { odr0, odr1 });
        let rcc_ahb2enr_gpioen = rcc_ahb2enr_gpioen.into_copy();
        if false {
            let mut val = gpio_odr.load().val();
            gpio_odr.odr0().set(&mut val);
            gpio_odr.store_val(val);
            rcc_ahb2enr_gpioen.read_bit();
        }
    }
    let reg = unsafe { Regs::take() };
    let gpio_c = periph_gpio_c!(reg);
    f(gpio_c);
}
