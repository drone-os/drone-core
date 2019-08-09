//! Memory-mapped registers.
//!
//! # Mappings and Tokens
//!
//! Most register should be already mapped by platform crates.
//!
//! ```
//! # #![feature(prelude_import)]
//! # #![feature(proc_macro_hygiene)]
//! # #[prelude_import] use drone_core::prelude::*;
//! use core::mem::size_of_val;
//! use drone_core::{reg::prelude::*, token::Tokens};
//!
//! drone_core::reg! {
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
//! drone_core::reg::unsafe_tokens! {
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
//!     let reg = unsafe { Regs::take() };
//!     assert_eq!(size_of_val(&reg.stk_ctrl.enable), 0);
//!     assert_eq!(size_of_val(&reg.stk_ctrl), 0);
//!     assert_eq!(size_of_val(&reg), 0);
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

mod compile_tests {
    //! ```compile_fail
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg! {
    //!     pub mod TST TST_RW_REG;
    //!     0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg;
    //!     TST_BIT { 0 1 RRRegField WWRegField }
    //! }
    //! fn assert_rw_reg_unsync<'a, T: drone_core::reg::RwRegUnsync<'a>>() {}
    //! fn main() {
    //!     assert_rw_reg_unsync::<tst_tst_rw_reg::Reg<Srt>>();
    //! }
    //! ```
    //!
    //! ```compile_fail
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg! {
    //!     pub mod TST TST_RO_REG;
    //!     0xDEAD_BEEF 0x20 0xBEEF_CACE RReg RoReg;
    //!     TST_BIT { 0 1 RRRegField RoRRegField }
    //! }
    //! fn assert_rw_reg_unsync<'a, T: drone_core::reg::RwRegUnsync<'a>>() {}
    //! fn main() {
    //!     assert_rw_reg_unsync::<tst_tst_ro_reg::Reg<Urt>>();
    //! }
    //! ```
    //!
    //! ```compile_fail
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg! {
    //!     pub mod TST TST_WO_REG;
    //!     0xDEAD_BEEF 0x20 0xBEEF_CACE WReg WoReg;
    //!     TST_BIT { 0 1 WWRegField WoWRegField }
    //! }
    //! fn assert_rw_reg_unsync<'a, T: drone_core::reg::RwRegUnsync<'a>>() {}
    //! fn main() {
    //!     assert_rw_reg_unsync::<tst_tst_wo_reg::Reg<Urt>>();
    //! }
    //! ```
    //!
    //! ```
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg! {
    //!     pub mod FOO BAR;
    //!     0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg;
    //!     BAZ { 0 1 RRRegField WWRegField }
    //! }
    //! fn assert_rw_reg_unsync<'a, T: drone_core::reg::RwRegUnsync<'a>>() {}
    //! fn main() {
    //!     assert_rw_reg_unsync::<foo_bar::Reg<Urt>>();
    //! }
    //! ```
    //!
    //! ```compile_fail
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg!(pub mod FOO BAR; 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 });
    //! fn assert_copy<T: Copy>() {}
    //! fn main() {
    //!     assert_copy::<foo_bar::Reg<Urt>>();
    //! }
    //! ```
    //!
    //! ```compile_fail
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg!(pub mod FOO BAR; 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 });
    //! fn assert_clone<T: Clone>() {}
    //! fn main() {
    //!     assert_clone::<foo_bar::Reg<Urt>>();
    //! }
    //! ```
    //!
    //! ```compile_fail
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg!(pub mod FOO BAR; 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 });
    //! fn assert_copy<T: Copy>() {}
    //! fn main() {
    //!     assert_copy::<foo_bar::Reg<Srt>>();
    //! }
    //! ```
    //!
    //! ```compile_fail
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg!(pub mod FOO BAR; 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 });
    //! fn assert_clone<T: Clone>() {}
    //! fn main() {
    //!     assert_clone::<foo_bar::Reg<Srt>>();
    //! }
    //! ```
    //!
    //! ```
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg!(pub mod FOO BAR; 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 });
    //! fn assert_copy<T: Copy>() {}
    //! fn main() {
    //!     assert_copy::<foo_bar::Reg<Crt>>();
    //! }
    //! ```
    //!
    //! ```
    //! use drone_core::reg::prelude::*;
    //! drone_core::reg!(pub mod FOO BAR; 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 });
    //! fn assert_clone<T: Clone>() {}
    //! fn main() {
    //!     assert_clone::<foo_bar::Reg<Crt>>();
    //! }
    //! ```
}
