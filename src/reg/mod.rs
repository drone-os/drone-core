//! Memory-mapped registers.
//!
//! # Binding
//!
//! To use a register one should bind it with the register macro. The macro
//! ensures each register to be bound not more than once across the whole
//! program.
//!
//! ```
//! # #![feature(decl_macro)]
//! # #![feature(linkage)]
//! # use std as core;
//! # pub mod reg { pub mod prelude { pub use drone::reg::prelude::*; } }
//! # drone::reg!(STK CTRL { 0xE000_E010 0x20 0x0000_0000 ENABLE { 0 1 } });
//! use drone::reg::prelude::*;
//! use core::mem::size_of_val;
//!
//! fn main() {
//!   let stk_ctrl = stk::Ctrl!(Ur);
//!   assert_eq!(size_of_val(&stk_ctrl), 0);
//! }
//! ```
//!
//! # Mapping
//!
//! Most of registers should be already mapped by platform crates. These crates
//! should map registers with `reg!` macro as follows:
//!
//! ```
//! # #![feature(decl_macro)]
//! # fn main() {}
//! use drone::reg;
//! use drone::reg::prelude::*;
//!
//! reg! {
//!   //! SysTick timer.
//!   STK // block name
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
//! ```

pub mod prelude;

mod field;
mod hold;
mod raw;
#[cfg_attr(feature = "clippy", allow(module_inception))]
mod reg;
mod tag;
mod val;

pub use self::field::*;
pub use self::hold::*;
pub use self::raw::*;
pub use self::reg::*;
pub use self::tag::*;
pub use self::val::*;
