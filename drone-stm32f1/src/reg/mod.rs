//! *STM32F1* peripheral memory map.

use core::marker::PhantomData;
use drone_core::reg::{RawAlias, RawPointer, RawValue, RegionAlias};

/// Peripheral bit-banding alias region base.
pub const PERIPHERAL_ALIAS_BASE: usize = 0x4200_0000;

pub mod dbg;
pub mod flash;
pub mod gpio;
pub mod rcc;
pub mod scb;
pub mod stk;

define_reg_structs!();
