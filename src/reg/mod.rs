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
//! use drone_core::{
//!   reg::{self, prelude::*},
//!   token::Tokens,
//! };
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
//! reg::unsafe_tokens! {
//!   /// Defines an index of register tokens.
//!   ///
//!   /// # Safety
//!   ///
//!   /// See [`::drone_core::reg::unsafe_tokens!`].
//!   pub macro unsafe_reg_tokens;
//!   super;;
//!
//!   /// SysTick timer.
//!   pub mod STK {
//!     CTRL;
//!   }
//! }
//!
//! unsafe_reg_tokens! {
//!   /// Register tokens.
//!   pub struct Regs;
//! }
//!
//! fn main() {
//!   let reg = unsafe { Regs::take() };
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
pub use drone_core_macros::unsafe_reg_tokens as unsafe_tokens;
