//! Memory-mapped registers.
//!
//! # Mapping
//!
//! Most of registers should be already mapped by platform crates. These crates
//! should map registers with [`reg!`] macro as follows:
//!
//! ```
//! # #![feature(decl_macro)]
//! # fn main() {}
//! use drone::{reg, reg_block};
//! use drone::reg::prelude::*;
//!
//! reg_block! {
//!   //! SysTick timer.
//!   STK // peripheral name
//!
//!   reg! {
//!     //! SysTick control and status register.
//!     CTRL // register name
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
//!
//! # Binding
//!
//! It is strongly encouraged to bind registers with a single [`bind!`] block at
//! the very beginning of the application entry point.
//!
//! ```
//! # #![feature(decl_macro)]
//! # use std as core;
//! # drone::reg_block! {
//! #   STK
//! #   reg! {
//! #     CTRL 0xE000_E010 0x20 0x0000_0000 RReg WReg
//! #     ENABLE { 0 1 RRegField WRegField }
//! #   }
//! # }
//! # fn main() {
//! use drone::reg;
//! use drone::reg::prelude::*;
//! use core::mem::size_of_val;
//!
//! reg::bind! {
//!   stk_ctrl: stk::Ctrl<Ur>,
//! }
//!
//! // Use the bindings inside the current scope.
//! assert_eq!(size_of_val(&stk_ctrl), 0);
//! # }
//! ```
//!
//! [`bind!`]: ../macro.bind.html
//! [`reg!`]: ../macro.reg.html

pub mod prelude;

#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod field;
#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod hold;
#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod raw;
#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
#[cfg_attr(feature = "clippy", allow(module_inception))]
mod reg;
#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod tag;
#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod val;

pub use self::field::*;
pub use self::hold::*;
pub use self::raw::*;
pub use self::reg::*;
pub use self::tag::*;
pub use self::val::*;
pub use drone_macros::{bind_impl, reg_block_impl, reg_impl};

/// Binds memory-mapped registers.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: reg/index.html
pub macro bind($($tokens:tt)*) {
  $crate::reg::bind_impl!($($tokens)*);
}

/// Defines a memory-mapped register.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: reg/index.html
pub macro reg($($tokens:tt)*) {
  $crate::reg::reg_impl!($($tokens)*);
}

/// Defines a block of memory-mapped registers.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: reg/index.html
pub macro reg_block($($tokens:tt)*) {
  $crate::reg::reg_block_impl!($($tokens)*);
}
