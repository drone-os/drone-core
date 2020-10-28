#![feature(proc_macro_hygiene)]

use drone_core::{bitfield::Bitfield, reg::prelude::*, token::Token};
use std::mem::{size_of, size_of_val};

use drone_core::reg;

reg! {
    /// Provides identification information for the processor.
    pub mod SCB CPUID;
    0xE000_ED00 0x20 0x410F_C241
    RReg RoReg;
    /// Implementer code assigned by ARM.
    IMPLEMENTER { 24 8 RRRegField RoRRegField }
    /// Variant number.
    VARIANT { 20 4 RRRegField RoRRegField }
    /// Reads as `0xF`.
    ARCHITECTURE { 16 4 RRRegField RoRRegField }
    /// Part number of the processor.
    PARTNO { 4 12 RRRegField RoRRegField }
    /// Revision number.
    REVISION { 0 4 RRRegField RoRRegField }
}

reg! {
    /// Capture/Compare mode register 1. (input mode)
    pub mod TIM1 CCMR1_Input;
    0x4001_0018 0x20 0x0000_0000
    RReg WReg;
    /// Input Capture 1 filter.
    IC1F { 12 4 RRRegField WWRegField }
    /// Input Capture 1 prescaler.
    IC1PSC { 10 2 RRRegField WWRegField }
    /// Capture/Compare 1 selection.
    CC1S { 8 2 RRRegField WWRegField }

    /// Capture/Compare mode register 1. (output mode)
    pub mod TIM1 CCMR1_Output;
    0x4001_0018 0x20 0x0000_0000
    RReg WReg;
    /// Output Compare 1 clear enable.
    OC1CE { 15 1 RRRegField WWRegField }
    /// Output Compare 1 mode.
    OC1M { 12 3 RRRegField WWRegField }
    /// Output Compare 1 preload enable.
    OC1PE { 11 1 RRRegField WWRegField }
    /// Output Compare 1 fast enable.
    OC1FE { 10 1 RRRegField WWRegField }
    /// Capture/Compare 1 selection.
    CC1S { 8 2 RRRegField WWRegField }
}

reg::tokens! {
    /// Intermediate register tokens macro.
    pub macro reg_tokens_intermediate;
    crate;
    crate;

    /// System control block.
    pub mod SCB {
        CPUID;
    }
}

reg::tokens! {
    /// Register tokens macro.
    pub macro reg_tokens;
    use macro reg_tokens_intermediate;
    crate;
    crate;

    /// Advanced-timer.
    pub mod TIM1 {
        CCMR1_Input;
        !CCMR1_Output;
    }
}

reg_tokens! {
    /// Register tokens.
    index => pub Regs;
}

#[test]
fn default_val() {
    assert_eq!(unsafe { scb::Cpuid::<Srt>::take() }.default_val().bits(), 0x410F_C241);
}

#[test]
fn sizes() {
    assert_eq!(size_of::<Regs>(), 0);
    assert_eq!(size_of::<scb::Cpuid<Urt>>(), 0);
    assert_eq!(size_of::<scb::Cpuid<Srt>>(), 0);
    assert_eq!(size_of::<scb::Cpuid<Crt>>(), 0);
    assert_eq!(size_of::<scb::cpuid::Val>(), 4);
}

#[test]
fn tokens() {
    let reg = unsafe { Regs::take() };
    assert_eq!(size_of_val(&reg.scb_cpuid), 0);
    assert_eq!(size_of_val(&reg.tim1_ccmr1_input), 0);
}

#[test]
fn variants() {
    let input: tim1::Ccmr1Input<Srt> = unsafe { Token::take() };
    let output: tim1::Ccmr1Output<Srt> = input.into_tim1_ccmr1_output();
    let _input: tim1::Ccmr1Input<Srt> = output.into_tim1_ccmr1_input();
}
