#![feature(proc_macro_hygiene)]

use drone_core::{
    reg,
    reg::{marker::*, prelude::*},
    token::Token,
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

reg! {
    pub mod TIM1 CCMR1_Output;
    0 32 0 RReg WReg;
    OC1CE { 7 1 RRRegField WWRegField }
    OC1M { 4 3 RRRegField WWRegField }
    OC1PE { 3 1 RRRegField WWRegField }
    OC1FE { 2 1 RRRegField WWRegField }
    CC1S { 0 2 RRRegField WWRegField }
    pub mod TIM1 CCMR1_Input;
    0 32 0 RReg WReg;
    IC1F { 4 4 RRRegField WWRegField }
    IC1PSC { 2 2 RRRegField WWRegField }
    CC1S { 0 2 RRRegField WWRegField }
}

reg! {
    pub mod TIM1 CCMR2_Output;
    0 32 0 RReg WReg;
    OC1CE { 7 1 RRRegField WWRegField }
    OC1M { 4 3 RRRegField WWRegField }
    OC1PE { 3 1 RRRegField WWRegField }
    OC1FE { 2 1 RRRegField WWRegField }
    CC1S { 0 2 RRRegField WWRegField }
    pub mod TIM1 CCMR2_Input;
    0 32 0 RReg WReg;
    IC1F { 4 4 RRRegField WWRegField }
    IC1PSC { 2 2 RRRegField WWRegField }
    CC1S { 0 2 RRRegField WWRegField }
}

reg! {
    pub mod TIM2 CCMR1_Output;
    0 32 0 RReg WReg;
    OC1CE { 7 1 RRRegField WWRegField }
    OC1M { 4 3 RRRegField WWRegField }
    OC1PE { 3 1 RRRegField WWRegField }
    OC1FE { 2 1 RRRegField WWRegField }
    CC1S { 0 2 RRRegField WWRegField }
    pub mod TIM2 CCMR1_Input;
    0 32 0 RReg WReg;
    IC1F { 4 4 RRRegField WWRegField }
    IC1PSC { 2 2 RRRegField WWRegField }
    CC1S { 0 2 RRRegField WWRegField }
}

reg! {
    pub mod TWIM0_NS TASKS_STARTTX;
    0 32 0 WReg WoReg;
    TASKS_STARTTX { 0 1 WWRegField WoWRegField }
    pub mod UARTE0_NS TASKS_STARTTX;
    0 32 0 WReg WoReg;
    TASKS_STARTTX { 0 1 WWRegField WoWRegField }
}

reg::tokens! {
    pub macro reg_tokens;
    crate;
    crate;

    pub mod RCC { AHB2ENR; }
    pub mod GPIOA { ODR; IDR; }
    pub mod GPIOB { ODR; IDR; }
    pub mod GPIOC { ODR; }
    pub mod TIM1 { CCMR1_Output; !CCMR1_Input; CCMR2_Output; !CCMR2_Input; }
    pub mod TIM2 { CCMR1_Output; !CCMR1_Input; }
    pub mod TWIM0_NS { TASKS_STARTTX; }
    pub mod UARTE0_NS { !TASKS_STARTTX; }
}

reg_tokens! {
    index => pub Regs;
}

pub mod gpio {
    use drone_core::{periph, reg::marker::*};

    periph! {
        pub trait GpioMap {}
        pub struct GpioPeriph;

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
        super;
        crate::gpio;

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
        super;
        crate::gpio;

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
        super;
        crate::gpio;

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
}

pub mod tim {
    use drone_core::{periph, reg::marker::*};

    periph! {
        pub trait TimMap {}
        pub struct TimPeriph;

        TIM {
            CCMR1 {
                @Output 0x20 RwReg;
                CC1S { RwRwRegFieldBits }
                OC1CE { RwRwRegFieldBit }
                OC1FE { RwRwRegFieldBit }
                OC1M { RwRwRegFieldBits }
                OC1PE { RwRwRegFieldBit }
                @Input 0x20 RwReg;
                CC1S { RwRwRegFieldBits }
                IC1F { RwRwRegFieldBits }
                IC1PSC { RwRwRegFieldBits }
            }
            CCMR2 {
                @Output 0x20 RwReg Option;
                CC1S { RwRwRegFieldBits }
                OC1CE { RwRwRegFieldBit }
                OC1FE { RwRwRegFieldBit }
                OC1M { RwRwRegFieldBits }
                OC1PE { RwRwRegFieldBit }
                @Input 0x20 RwReg Option;
                CC1S { RwRwRegFieldBits }
                IC1F { RwRwRegFieldBits }
                IC1PSC { RwRwRegFieldBits }
            }
        }
    }

    periph::map! {
        pub macro periph_tim1;
        pub struct Tim1;
        impl TimMap for Tim1 {}
        super;
        crate::tim;

        TIM {
            TIM1;
            CCMR1 {
                @Output CCMR1_Output;
                CC1S { CC1S }
                OC1CE { OC1CE }
                OC1FE { OC1FE }
                OC1M { OC1M }
                OC1PE { OC1PE }
                @Input CCMR1_Input;
                CC1S { CC1S }
                IC1F { IC1F }
                IC1PSC { IC1PSC }
            }
            CCMR2 {
                @Output CCMR2_Output Option;
                CC1S { CC1S }
                OC1CE { OC1CE }
                OC1FE { OC1FE }
                OC1M { OC1M }
                OC1PE { OC1PE }
                @Input CCMR2_Input Option;
                CC1S { CC1S }
                IC1F { IC1F }
                IC1PSC { IC1PSC }
            }
        }
    }

    periph::map! {
        pub macro periph_tim2;
        pub struct Tim2;
        impl TimMap for Tim2 {}
        super;
        crate::tim;

        TIM {
            TIM2;
            CCMR1 {
                @Output CCMR1_Output;
                CC1S { CC1S }
                OC1CE { OC1CE }
                OC1FE { OC1FE }
                OC1M { OC1M }
                OC1PE { OC1PE }
                @Input CCMR1_Input;
                CC1S { CC1S }
                IC1F { IC1F }
                IC1PSC { IC1PSC }
            }
            CCMR2 {
                @Output
                @Input
            }
        }
    }
}

pub mod uarte {
    use drone_core::{periph, reg::marker::*};

    periph! {
        pub trait UarteMap {}
        pub struct UartePeriph;

        UARTE {
            TASKS_STARTTX {
                0x20 WoReg;
                TASKS_STARTTX { WoWoRegFieldBit }
            }
        }
    }

    periph::map! {
        pub macro periph_uarte0_ns;
        pub struct Uarte0Ns;
        impl UarteMap for Uarte0Ns {}
        super;
        crate::uarte;

        UARTE {
            UARTE0_NS;
            TASKS_STARTTX {
                TASKS_STARTTX(TWIM0_NS TASKS_STARTTX);
                TASKS_STARTTX { TASKS_STARTTX }
            }
        }
    }
}

#[test]
fn periph_macros() {
    #![allow(unused_variables)]
    let reg = unsafe { Regs::take() };
    let gpioa = periph_gpio_a!(reg);
    let gpiob = periph_gpio_b!(reg);
    let gpioc = periph_gpio_c!(reg);
    let tim1 = periph_tim1!(reg);
    let tim2 = periph_tim2!(reg);
    let uarte0_ns = periph_uarte0_ns!(reg);
}

#[test]
fn concrete() {
    use gpio::*;
    let reg = unsafe { Regs::take() };
    let gpio_c = periph_gpio_c!(reg);
    let GpioPeriph { rcc_ahb2enr_gpioen, rcc_ahb2enr_gpiorst: (), gpio_odr, gpio_idr: () } = gpio_c;
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
    use gpio::*;
    fn f<T: GpioMap + GpioOdrOdr1 + GpioIdr + GpioIdrIdr1>(gpio: GpioPeriph<T>) {
        let GpioPeriph { rcc_ahb2enr_gpioen, rcc_ahb2enr_gpiorst: _, gpio_odr, gpio_idr: _ } = gpio;
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
    use gpio::*;
    fn f<T: GpioMap>(gpio: GpioPeriph<T>) {
        let GpioPeriph { rcc_ahb2enr_gpioen, rcc_ahb2enr_gpiorst: _, gpio_odr, gpio_idr: _ } = gpio;
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

#[test]
fn variants() {
    use tim::*;
    let reg = unsafe { Regs::take() };
    let tim1 = periph_tim1!(reg);
    let tim2 = periph_tim2!(reg);
    let TimPeriph { tim_ccmr1_output: tim1_ccmr1_output, tim_ccmr2_output: tim1_ccmr2_output } =
        tim1;
    let tim1_ccmr1_input = tim1_ccmr1_output.into_input();
    let _tim1_ccmr1_output = tim1_ccmr1_input.into_output();
    let tim1_ccmr2_input = tim1_ccmr2_output.into_input();
    let _tim1_ccmr2_output = tim1_ccmr2_input.into_output();
    let TimPeriph { tim_ccmr1_output: tim2_ccmr1_output, tim_ccmr2_output: () } = tim2;
    let tim2_ccmr1_input = tim2_ccmr1_output.into_input();
    let _tim2_ccmr1_output = tim2_ccmr1_input.into_output();
}

#[test]
fn generic_variants_with_holes() {
    use tim::*;
    fn f<T: TimMap>(tim: TimPeriph<T>) {
        let TimPeriph { tim_ccmr1_output, tim_ccmr2_output: _ } = tim;
        let tim_ccmr1_input = tim_ccmr1_output.into_input();
        let _tim_ccmr1_output = tim_ccmr1_input.into_output();
    }
    let reg = unsafe { Regs::take() };
    let tim2 = periph_tim2!(reg);
    f(tim2);
}

#[test]
fn generic_variants_without_holes() {
    use tim::*;
    fn f<T: TimMap + TimCcmr2Output + TimCcmr2Input>(tim: TimPeriph<T>) {
        let TimPeriph { tim_ccmr1_output, tim_ccmr2_output } = tim;
        let tim_ccmr1_input = tim_ccmr1_output.into_input();
        let _tim_ccmr1_output = tim_ccmr1_input.into_output();
        let tim_ccmr2_input = tim_ccmr2_output.into_input();
        let _tim_ccmr2_output = tim_ccmr2_input.into_output();
    }
    let reg = unsafe { Regs::take() };
    let tim1 = periph_tim1!(reg);
    f(tim1);
}
