//! Memory-mapped registers.
//!
//! # Mapping and Binding
//!
//! Most register should be already mapped by platform crates.
//!
//! ```
//! # #![feature(decl_macro)]
//! # use std as core;
//! use core::mem::size_of_val;
//! use drone::reg::{bindings, mappings};
//! use drone::reg::prelude::*;
//!
//! mappings! {
//!   /// SysTick timer.
//!   STK; // block name
//!
//!   /// SysTick control and status register.
//!   CTRL { // register name
//!     0xE000_E010 // memory address
//!     0x20 // bit size
//!     0x0000_0000 // reset value
//!     RReg WReg // list of marker traits for the register
//!
//!     /// Counter enable.
//!     ENABLE { // field name
//!       0 // offset
//!       1 // width
//!       RRegField WRegField // list of marker traits for the field
//!     }
//!   }
//! }
//!
//! bindings! {
//!   /// Register bindings.
//!   Bindings;
//!
//!   STK {
//!     /// SysTick control and status register.
//!     CTRL;
//!   }
//! }
//!
//! fn main() {
//!   let bindings = unsafe { Bindings::new() };
//!   assert_eq!(size_of_val(&bindings.stk_ctrl.enable), 0);
//!   assert_eq!(size_of_val(&bindings.stk_ctrl), 0);
//!   assert_eq!(size_of_val(&bindings), 0);
//! }
//! ```

pub mod prelude;

mod bindings;
mod field;
mod hold;
mod raw;
#[cfg_attr(feature = "clippy", allow(module_inception))]
mod reg;
mod tag;
mod val;

pub use self::bindings::*;
pub use self::field::*;
pub use self::hold::*;
pub use self::raw::*;
pub use self::reg::*;
pub use self::tag::*;
pub use self::val::*;
pub use drone_macros::{reg_bindings as bindings, reg_mappings as mappings};

/// Forkable binding.
pub trait RegFork {
  /// Returns a duplicate of the binding.
  fn fork(&mut self) -> Self;
}
