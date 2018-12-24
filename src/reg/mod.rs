//! Memory-mapped registers.
//!
//! # Mappings and Tokens
//!
//! Most register should be already mapped by platform crates.
//!
//! ```
//! # #![feature(prelude_import)]
//! # #![feature(proc_macro_hygiene)]
//! # use std as core;
//! # #[macro_use] extern crate drone_core;
//! # #[prelude_import] use drone_core::prelude::*;
//! use core::mem::size_of_val;
//! use drone_core::reg::{self, prelude::*};
//!
//! reg! {
//!   /// SysTick control and status register.
//!   pub mod STK CTRL; // register block and name
//!
//!   0xE000_E010 // memory address
//!   0x20 // bit size
//!   0x0000_0000 // reset value
//!   RReg WReg; // list of marker traits for the register
//!
//!   /// Counter enable.
//!   ENABLE { // field name
//!     0 // offset
//!     1 // width
//!     RRRegField WWRegField // list of marker traits for the field
//!   }
//! }
//!
//! reg::index! {
//!   /// Define register tokens.
//!   pub macro reg_idx;
//!   super;;
//!
//!   /// SysTick timer.
//!   pub mod STK {
//!     CTRL;
//!   }
//! }
//!
//! reg_idx! {
//!   /// Register tokens.
//!   pub struct RegIdx;
//! }
//!
//! fn main() {
//!   let reg = unsafe { RegIdx::new() };
//!   assert_eq!(size_of_val(&reg.stk_ctrl.enable), 0);
//!   assert_eq!(size_of_val(&reg.stk_ctrl), 0);
//!   assert_eq!(size_of_val(&reg), 0);
//! }
//! ```

pub mod marker;
pub mod prelude;

mod field;
mod hold;
mod reg;
mod tag;

pub use self::{field::*, hold::*, reg::*, tag::*};
pub use drone_core_macros::reg_index as index;

/// An index of register tokens.
pub trait RegTokens {
  /// Creates a new set of register tokens.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Register tokens belonging to the set must not overlap.
  unsafe fn new() -> Self;
}
