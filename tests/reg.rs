#![feature(proc_macro_hygiene)]
#![no_implicit_prelude]

use ::drone_core::bitfield::Bitfield;
use ::drone_core::reg;
use ::drone_core::reg::prelude::*;
use ::drone_core::token::Token;
use ::std::assert_eq;
use ::std::mem::{size_of, size_of_val};

reg! {
    /// Provides identification information for the processor.
    pub SCB CPUID => {
        address => 0xE000_ED00;
        size => 0x20;
        reset => 0x410F_C241;
        traits => { RReg RoReg };
        fields => {
            /// Implementer code assigned by ARM.
            IMPLEMENTER => {
                offset => 24;
                width => 8;
                traits => { RRRegField RoRRegField };
            };
            /// Variant number.
            VARIANT => {
                offset => 20;
                width => 4;
                traits => { RRRegField RoRRegField };
            };
            /// Reads as `0xF`.
            ARCHITECTURE => {
                offset => 16;
                width => 4;
                traits => { RRRegField RoRRegField };
            };
            /// Part number of the processor.
            PARTNO => {
                offset => 4;
                width => 12;
                traits => { RRRegField RoRRegField };
            };
            /// Revision number.
            REVISION => {
                offset => 0;
                width => 4;
                traits => { RRRegField RoRRegField };
            };
        };
    };
}

reg! {
    /// Capture/Compare mode register 1. (input mode)
    pub TIM1 CCMR1_Input => {
        address => 0x4001_0018;
        size => 0x20;
        reset => 0x0000_0000;
        traits => { RReg WReg };
        fields => {
            /// Input Capture 1 filter.
            IC1F => {
                offset => 12;
                width => 4;
                traits => { RRRegField WWRegField };
            };
            /// Input Capture 1 prescaler.
            IC1PSC => {
                offset => 10;
                width => 2;
                traits => { RRRegField WWRegField };
            };
            /// Capture/Compare 1 selection.
            CC1S => {
                offset => 8;
                width => 2;
                traits => { RRRegField WWRegField };
            };
        };
    };

    /// Capture/Compare mode register 1. (output mode)
    pub TIM1 CCMR1_Output => {
        address => 0x4001_0018;
        size => 0x20;
        reset => 0x0000_0000;
        traits => { RReg WReg };
        fields => {
            /// Output Compare 1 clear enable.
            OC1CE => {
                offset => 15;
                width => 1;
                traits => { RRRegField WWRegField };
            };
            /// Output Compare 1 mode.
            OC1M => {
                offset => 12;
                width => 3;
                traits => { RRRegField WWRegField };
            };
            /// Output Compare 1 preload enable.
            OC1PE => {
                offset => 11;
                width => 1;
                traits => { RRRegField WWRegField };
            };
            /// Output Compare 1 fast enable.
            OC1FE => {
                offset => 10;
                width => 1;
                traits => { RRRegField WWRegField };
            };
            /// Capture/Compare 1 selection.
            CC1S => {
                offset => 8;
                width => 2;
                traits => { RRRegField WWRegField };
            };
        };
    };
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
