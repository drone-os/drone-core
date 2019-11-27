//! The Memory-Mapped Registers module.
//!
//! **NOTE** A Drone platform crate may re-export this module with its own
//! additions under the same name, in which case it should be used instead.
//!
//! A memory-mapped register is a special location in memory. Reads and writes
//! from/to this location produce side-effects. For example writing `1` or `0`
//! to such location may set the related GPIO output pin to the high or low
//! logical level. In the same way reading from such location may return `1` or
//! `0` depending on the input level of the related GPIO input pin. The most
//! basic way to deal with this memory is to use [`core::ptr::read_volatile`]
//! and [`core::ptr::write_volatile`]. Here is an example:
//!
//! ```no_run
//! use core::ptr::{read_volatile, write_volatile};
//!
//! // The address of GPIOA_CRL register.
//! const GPIOA_CRL: usize = 0x4001_0800;
//! // The offset for MODE2 field of GPIOA_CRL register.
//! const GPIOA_CRL_MODE2_OFFSET: usize = 8;
//! // The mask for MODE2 field of GPIOA_CRL register.
//! const GPIOA_CRL_MODE2_MASK: u32 = 0x0000_0300;
//!
//! // Read the state of GPIOA_CRL register. The function is unsafe because it
//! // can read from any location in memory.
//! let mut gpioa_crl = unsafe { read_volatile(GPIOA_CRL as *const u32) };
//! // Do bit arithmetic to get the value of MODE2 field.
//! let mut gpioa_crl_mode2 = (gpioa_crl & GPIOA_CRL_MODE2_MASK) >> GPIOA_CRL_MODE2_OFFSET;
//! // Toggle some bits.
//! gpioa_crl_mode2 ^= 0b10;
//! // Do bit arithmetic to update the register value with the new field value.
//! gpioa_crl = gpioa_crl & !GPIOA_CRL_MODE2_MASK | gpioa_crl_mode2 << GPIOA_CRL_MODE2_OFFSET;
//! // Update the state of GPIOA_CRL register. The function is also unsafe
//! // because it can write to any location in memory.
//! unsafe { write_volatile(GPIOA_CRL as *mut u32, gpioa_crl) };
//! ```
//!
//! This way has numerous disadvantages: it's obscure, verbose, error-prone,
//! and requires `unsafe` blocks. Also it has less obvious problems like lack of
//! thread-safety. This module provides safe and zero-cost abstractions to
//! this problem. As result the above example can be written like this:
//!
//! ```no_run
//! # use drone_core::{reg::prelude::*, token::Token};
//! # drone_core::reg! {
//! #     mod GPIOA CRL; 0x4001_0800 0x20 0 RReg WReg;
//! #     MODE2 { 8 2 RRRegField WWRegField }
//! # }
//! # fn main() {
//! #   let mut gpioa_crl = unsafe { gpioa_crl::Reg::<Urt>::take() };
//! gpioa_crl.modify(|r| r.write_mode2(r.mode2() ^ 0b10));
//! # }
//! ```
//!
//! We abstract this type of memory with zero-sized [`token`]s. (Familiarity
//! with [`token`] module is required.) Only the code that have the token
//! instance for a particular memory-mapped register can manipulate it safely.
//! At the very base there is *Register Field Token* (like `MODE2` in the above
//! example.) Register Field Tokens for a particular register grouped in
//! *Register Token* (like `GPIO_CRL` in the above example.) And all available
//! Register Tokens are grouped in one *Register Tokens Index*.
//!
//! # API
//!
//! The memory-mapped registers API is scattered across numerous traits.
//! Therefore it is recommended to use [`reg::prelude`]:
//!
//! ```
//! # #![allow(unused_imports)]
//! use drone_core::reg::prelude::*;
//! ```
//!
//! ## Field Token
//!
//! |                                                | Field Width | Field Mode | Register Mode |
//! |----------------------------------------------------------|-----------|-------|------------|
//! | [`into_unsync`](reg::field::RegField::into_unsync)       |           |       |            |
//! | [`into_sync`](reg::field::RegField::into_sync)           |           |       |            |
//! | [`into_copy`](reg::field::RegField::into_copy)           |           |       |            |
//! | [`as_sync`](reg::field::RegField::as_sync)               |           |       |            |
//! | [`load_val`](reg::field::RRRegField::load_val)           |           | read  | read       |
//! | [`default_val`](reg::field::WoWoRegField::default_val)   |           | write | write-only |
//! | [`store_val`](reg::field::WoWoRegField::store_val)       |           | write | write-only |
//! | [`store`](reg::field::WoWoRegField::store)               |           | write | write-only |
//! | [`read`](reg::field::RRRegFieldBit::read)                | one-bit   | read  | read       |
//! | [`read_bit`](reg::field::RRRegFieldBit::read_bit)        | one-bit   | read  | read       |
//! | [`set`](reg::field::WWRegFieldBit::set)                  | one-bit   | write | write      |
//! | [`clear`](reg::field::WWRegFieldBit::clear)              | one-bit   | write | write      |
//! | [`toggle`](reg::field::WWRegFieldBit::toggle)            | one-bit   | write | write      |
//! | [`set_bit`](reg::field::WoWoRegFieldBit::set_bit)        | one-bit   | write | write-only |
//! | [`clear_bit`](reg::field::WoWoRegFieldBit::clear_bit)    | one-bit   | write | write-only |
//! | [`toggle_bit`](reg::field::WoWoRegFieldBit::toggle_bit)  | one-bit   | write | write-only |
//! | [`read`](reg::field::RRRegFieldBits::read)               | multi-bit | read  | read       |
//! | [`read_bits`](reg::field::RRRegFieldBits::read_bits)     | multi-bit | read  | read       |
//! | [`write`](reg::field::WWRegFieldBits::write)             | multi-bit | write | write      |
//! | [`write_bits`](reg::field::WoWoRegFieldBits::write_bits) | multi-bit | write | write-only |
//!
//! ## Register Token
//!
//! |                                             | Mode       | Tag      |
//! |---------------------------------------------|------------|----------|
//! | [`into_unsync`](reg::Reg::into_unsync)      |            |          |
//! | [`into_sync`](reg::Reg::into_sync)          |            |          |
//! | [`into_copy`](reg::Reg::into_copy)          |            |          |
//! | [`as_sync`](reg::Reg::as_sync)              |            |          |
//! | [`default_val`](reg::Reg::default_val)      |            |          |
//! | [`default`](reg::RegRef::default)           |            |          |
//! | [`hold`](reg::RegRef::hold)                 |            |          |
//! | [`load`](reg::RReg::load)                   | read       |          |
//! | [`load_val`](reg::RReg::load_val)           | read       |          |
//! | [`load_bits`](reg::RReg::load_bits)         | read       |          |
//! | [`as_ptr`](reg::RReg::as_ptr)               | read       |          |
//! | [`as_mut_ptr`](reg::WReg::as_mut_ptr)       | write      |          |
//! | [`store`](reg::WRegUnsync::store)           | write      | Urt      |
//! | [`store`](reg::WRegAtomic::store)           | write      | Srt, Crt |
//! | [`store_val`](reg::WRegUnsync::store_val)   | write      | Urt      |
//! | [`store_val`](reg::WRegAtomic::store_val)   | write      | Srt, Crt |
//! | [`store_bits`](reg::WRegUnsync::store_bits) | write      | Urt      |
//! | [`store_bits`](reg::WRegAtomic::store_bits) | write      | Srt, Crt |
//! | [`reset`](reg::WRegUnsync::reset)           | write      | Urt      |
//! | [`reset`](reg::WRegAtomic::reset)           | write      | Srt, Crt |
//! | [`modify`](reg::RwRegUnsync::modify)        | read-write | Urt      |
//!
//! ## Register Value
//!
//! Autogenerated field methods for [`RegHold`](reg::RegHold) (`foo` as an
//! example field name):
//!
//! |                                                                 | Field Width | Mode |
//! |------------------------------------------------------------------|-----------|-------|
//! | `foo()` ([`read`](reg::field::RRRegFieldBit::read))              | one-bit   | read  |
//! | `foo()` ([`read`](reg::field::RRRegFieldBits::read))             | multi-bit | read  |
//! | `set_foo()` ([`set`](reg::field::WWRegFieldBit::set))            | one-bit   | write |
//! | `clear_foo()` ([`clear`](reg::field::WWRegFieldBit::clear))      | one-bit   | write |
//! | `toggle_foo()` ([`toggle`](reg::field::WWRegFieldBit::toggle))   | one-bit   | write |
//! | `write_foo(bits)` ([`write`](reg::field::WWRegFieldBits::write)) | multi-bit | write |
//!
//! # Tags
//!
//! Each register or field token can have one of three flavors. They are encoded
//! by [`tag`](reg::tag)s in their types. For example `Reg<Urt>`, or
//! `RegField<Srt>`.
//!
//! Here are available tags and their properties:
//!
//! |                                         | Atomic | Affine |
//! |-----------------------------------------|--------|--------|
//! | [`Urt`](reg::tag::Urt) (Unsynchronized) | -      | **+**  |
//! | [`Srt`](reg::tag::Srt) (Synchronized)   | **+**  | **+**  |
//! | [`Crt`](reg::tag::Crt) (Copyable)       | **+**  | -      |
//!
//! **Atomic** means the token uses more costly atomic operations, but could be
//! shared between threads.
//!
//! **Non-atomic** means the token uses less costly non-atomic operations, but
//! couldn't be shared between threads.
//!
//! **Affine** means the token can't be cloned or copied and uses
//! move-semantics.
//!
//! **Non-affine** means the token could be freely copied.
//!
//! Tokens of some tags can be converted to the same tokens of other tags using
//! `.into_unsync()`, `.into_sync()`, `.into_copy()`. Here is the conversion
//! matrix for *register* tokens:
//!
//! | from \ to | Urt   | Srt   | Crt   |
//! |-----------|-------|-------|-------|
//! | Urt       | **+** | **+** | **+** |
//! | Srt       | **+** | **+** | **+** |
//! | Crt       | -     | -     | **+** |
//!
//! And here is the conversion matrix for *field* tokens:
//!
//! | from \ to | Urt   | Srt   | Crt   |
//! |-----------|-------|-------|-------|
//! | Urt       | **+** | -     | -     |
//! | Srt       | -     | **+** | **+** |
//! | Crt       | -     | -     | **+** |
//!
//! # Mappings
//!
//! We define concrete register mappings in platform crates. Usually the user
//! doesn't need to map registers themselves. But lets have a look to an example
//! of how it could be organized for STM32 platform:
//!
//! ```
//! # #![feature(proc_macro_hygiene)]
//!
//! use core::mem::size_of_val;
//! use drone_core::{reg::prelude::*, token::Token};
//!
//! use drone_core::reg;
//!
//! // ----- this is drone_cortex_m crate -----
//!
//! // Registers belong to blocks. Here we declare CTRL register in STK block.
//! reg! {
//!     // The output of this macro is `pub mod stk_ctrl { ... }`.
//!     /// SysTick control and status register.
//!     pub mod STK CTRL;
//!     0xE000_E010 // the register address in memory
//!     0x20        // size of the register in bits
//!     0x0000_0000 // reset value of the register
//!     // Traits to implement for the register token. The most common sets are:
//!     //     RReg RoReg - read-only register
//!     //     RReg WReg  - read-write register
//!     //     WReg WoReg - write-only register
//!     RReg WReg;
//!
//!     // Here we define register fields.
//!     /// Counter enable.
//!     ENABLE {
//!         0 // the offset of the field
//!         1 // the width of the field
//!         // Traits to implement for the field token. The most common sets are:
//!         //     RRRegField RoRRegField - read-only field
//!         //     RRRegField WWRegField  - read-write field
//!         //     WWRegField WoWRegField - read-write field
//!         RRRegField WWRegField
//!     }
//! }
//!
//! // Here we define the register tokens index. Actually the result of this macro
//! // is another macro, which can be used to define the final register token index
//! // or to extend with another registers in downstream crates. It will become
//! // clearer below.
//! reg::tokens! {
//!     // The result of this macro is
//!     // `macro_rules! cortex_m_reg_tokens { ... }`.
//!     /// Defines an index of core ARM Cortex-M register tokens.
//!     pub macro cortex_m_reg_tokens;
//!     // Path prefix to reach registers.
//!     crate;
//!     // Absolute path to the current module.
//!     crate;
//!
//!     // Here we declare all register blocks. This produces `pub mod stk { ... }`
//!     /// SysTick timer.
//!     pub mod STK {
//!         // Declare all registers for this block. This produces:
//!         // pub mod stk {
//!         //     pub use crate::stk_ctrl as ctrl;
//!         // }
//!         CTRL;
//!     }
//! }
//!
//! // ----- this is drone_stm32 crate -----
//! // This crate parses SVD files provided by the manufacturer and generates more
//! // registers.
//!
//! // Same as above, except it will reuse the upstream macro, resulting in a
//! // combined register tokens index. Note `use macro cortex_m_reg_tokens`.
//! reg::tokens! {
//!     /// Defines an index of STM32F103 register tokens.
//!     pub macro stm32_reg_tokens;
//!     use macro cortex_m_reg_tokens;
//!     crate;
//!     crate;
//! }
//!
//! // ----- this is an application crate -----
//!
//! // This macro defines the concrete register tokens index for STM32 MCU. The
//! // index is a sum of `drone_cortex_m` and `drone_stm32` registers. The result
//! // of this macro is `pub struct Regs { ... }`.
//! stm32_reg_tokens! {
//!     /// Register tokens.
//!     pub struct Regs;
//! }
//!
//! // Your entry point.
//! fn main() {
//!     // It's unsafe because we can accidentally create more than one instance
//!     // of the index.
//!     let reg = unsafe { Regs::take() };
//!     // The index doesn't really exist in memory.
//!     assert_eq!(size_of_val(&reg), 0);
//!     assert_eq!(size_of_val(&reg.stk_ctrl), 0);
//!     assert_eq!(size_of_val(&reg.stk_ctrl.enable), 0);
//!     // Pass the index to your safe entry point.
//!     trunk(reg);
//! }
//!
//! fn trunk(reg: Regs) {}
//! ```

pub mod field;
pub mod marker;
pub mod prelude;
pub mod tag;

/// A macro to define a macro to define a set of register tokens.
///
/// See [the module level documentation](self) for details.
#[doc(inline)]
pub use drone_core_macros::reg_tokens as tokens;

use self::tag::{Crt, RegAtomic, RegOwned, RegTag, Srt, Urt};
use crate::{bitfield::Bitfield, token::Token};
use core::ptr::{read_volatile, write_volatile};

/// The base trait for a memory-mapped register token.
pub trait Reg<T: RegTag>: Token + Sync {
    /// Opaque storage for register values.
    ///
    /// This type is only a storage, without methods to read or write the stored
    /// bits. It should be used in conjunction with [`RegHold`] or register
    /// [`field`]s.
    ///
    /// See also [`Hold`](RegRef::Hold).
    type Val: Bitfield;

    /// Corresponding unsynchronized register token.
    type UReg: Reg<Urt>;

    /// Corresponding synchronized register token.
    type SReg: Reg<Srt>;

    /// Corresponding copyable register token.
    type CReg: Reg<Crt>;

    /// The register address in memory.
    const ADDRESS: usize;

    /// The register default value.
    const RESET: <Self::Val as Bitfield>::Bits;

    /// Creates a new instance of [`Reg::Val`] from raw `bits`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't require a token.
    unsafe fn val_from(bits: <Self::Val as Bitfield>::Bits) -> Self::Val;

    /// Converts into unsynchronized register token.
    #[inline]
    fn into_unsync(self) -> Self::UReg
    where
        T: RegOwned,
    {
        unsafe { Self::UReg::take() }
    }

    /// Converts into synchronized register token.
    #[inline]
    fn into_sync(self) -> Self::SReg
    where
        T: RegOwned,
    {
        unsafe { Self::SReg::take() }
    }

    /// Converts into copyable register token.
    #[inline]
    fn into_copy(self) -> Self::CReg {
        unsafe { Self::CReg::take() }
    }

    /// Returns a reference to the synchronized register token.
    #[inline]
    fn as_sync(&self) -> &Self::SReg
    where
        T: RegAtomic,
    {
        unsafe { &*(self as *const Self as *const Self::SReg) }
    }

    /// Creates a new opaque register value, and initializes it with the reset
    /// value.
    ///
    /// See also [`default`](RegRef::default).
    #[inline]
    fn default_val(&self) -> Self::Val {
        unsafe { Self::val_from(Self::RESET) }
    }
}

/// Connects [`Reg`] with [`RegHold`].
pub trait RegRef<'a, T: RegTag>: Reg<T> {
    /// Exposed storage for register values.
    ///
    /// See also [`Val`](Reg::Val).
    type Hold: RegHold<'a, T, Self>;

    /// Creates a new exposed register value from `val`.
    fn hold(&'a self, val: Self::Val) -> Self::Hold;

    /// Creates a new exposed register value, and initializes it with the reset
    /// value.
    ///
    /// See also [`default_val`](Reg::default_val).
    #[inline]
    fn default(&'a self) -> Self::Hold {
        self.hold(self.default_val())
    }
}

/// Exposed storage for register values.
///
/// A type implementing this trait should have public getters and setters to
/// manipulate the protected data.
pub trait RegHold<'a, T, R>
where
    Self: Sized + 'a,
    T: RegTag,
    R: Reg<T>,
{
    /// Returns the opaque value.
    fn val(&self) -> R::Val;

    /// Replaces the opaque value.
    fn set_val(&mut self, val: R::Val);
}

/// Readable register.
pub trait RReg<T: RegTag>: Reg<T> {
    /// Reads the value from the register memory to the exposed value type.
    ///
    /// See also [`load_val`](RReg::load_val), [`load_bits`](RReg::load_bits).
    #[inline]
    fn load<'a>(&'a self) -> <Self as RegRef<'a, T>>::Hold
    where
        Self: RegRef<'a, T>,
    {
        self.hold(self.load_val())
    }

    /// Reads the value from the register memory to the opaque value type.
    ///
    /// See also [`load`](RReg::load), [`load_bits`](RReg::load_bits).
    #[inline]
    fn load_val(&self) -> Self::Val {
        unsafe { Self::val_from(self.load_bits()) }
    }

    /// Reads the value from the register memory to the raw value type.
    ///
    /// See also [`load`](RReg::load), [`load_val`](RReg::load_val).
    #[inline]
    fn load_bits(&self) -> <Self::Val as Bitfield>::Bits {
        unsafe { read_volatile(self.as_ptr()) }
    }

    /// Returns a raw pointer to the register memory.
    ///
    /// See also [`as_mut_ptr`](WReg::as_mut_ptr).
    #[inline]
    fn as_ptr(&self) -> *const <Self::Val as Bitfield>::Bits {
        Self::ADDRESS as *const <Self::Val as Bitfield>::Bits
    }
}

/// Writable register.
pub trait WReg<T: RegTag>: Reg<T> {
    /// Returns a mutable raw pointer to the register memory.
    ///
    /// See also [`as_ptr`](RReg::as_ptr).
    #[inline]
    fn as_mut_ptr(&self) -> *mut <Self::Val as Bitfield>::Bits {
        Self::ADDRESS as *mut <Self::Val as Bitfield>::Bits
    }
}

/// Read-only register.
pub trait RoReg<T: RegTag>: RReg<T> {}

/// Write-only register.
pub trait WoReg<T: RegTag>: WReg<T> {}

/// Non-atomic operations for writable register.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait WRegUnsync<'a>: WReg<Urt> + RegRef<'a, Urt> {
    /// Passes the reset value to the closure `f`, then writes the result of the
    /// closure into the register memory.
    ///
    /// See also [`store_val`](WRegUnsync::store_val),
    /// [`store_bits`](WRegUnsync::store_bits).
    fn store<F>(&'a mut self, f: F)
    where
        F: for<'b> FnOnce(
            &'b mut <Self as RegRef<'a, Urt>>::Hold,
        ) -> &'b mut <Self as RegRef<'a, Urt>>::Hold;

    /// Writes an opaque value `val` into the register memory.
    ///
    /// See also [`store`](WRegUnsync::store),
    /// [`store_bits`](WRegUnsync::store_bits).
    fn store_val(&mut self, val: Self::Val);

    /// Writes raw `bits` into the register memory.
    ///
    /// See also [`store`](WRegUnsync::store),
    /// [`store_val`](WRegUnsync::store_val).
    fn store_bits(&mut self, bits: <Self::Val as Bitfield>::Bits);

    /// Writes the reset value into the register memory.
    fn reset(&'a mut self);
}

/// Atomic operations for writable register.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait WRegAtomic<'a, T: RegAtomic>: WReg<T> + RegRef<'a, T> {
    /// Passes the reset value to the closure `f`, then writes the result of the
    /// closure into the register memory.
    ///
    /// See also [`store_val`](WRegAtomic::store_val),
    /// [`store_bits`](WRegAtomic::store_bits).
    fn store<F>(&'a self, f: F)
    where
        F: for<'b> FnOnce(
            &'b mut <Self as RegRef<'a, T>>::Hold,
        ) -> &'b mut <Self as RegRef<'a, T>>::Hold;

    /// Writes an opaque value `val` into the register memory.
    ///
    /// See also [`store`](WRegAtomic::store),
    /// [`store_bits`](WRegAtomic::store_bits).
    fn store_val(&self, val: Self::Val);

    /// Writes raw `bits` into the register memory.
    ///
    /// See also [`store`](WRegAtomic::store),
    /// [`store_val`](WRegAtomic::store_val).
    fn store_bits(&self, bits: <Self::Val as Bitfield>::Bits);

    /// Writes the reset value into the register memory.
    fn reset(&'a self);
}

/// Non-atomic operations for read-write register.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait RwRegUnsync<'a>: RReg<Urt> + WRegUnsync<'a> + RegRef<'a, Urt> {
    /// Reads the value from the register memory, then passes the value to the
    /// closure `f`, then writes the result of the closure back to the register
    /// memory.
    ///
    /// This operation is non-atomic, thus it requires a mutable reference to
    /// the token.
    fn modify<F>(&'a mut self, f: F)
    where
        F: for<'b> FnOnce(
            &'b mut <Self as RegRef<'a, Urt>>::Hold,
        ) -> &'b mut <Self as RegRef<'a, Urt>>::Hold;
}

impl<'a, R> WRegUnsync<'a> for R
where
    R: WReg<Urt> + RegRef<'a, Urt>,
{
    #[inline]
    fn store<F>(&'a mut self, f: F)
    where
        F: for<'b> FnOnce(
            &'b mut <Self as RegRef<'a, Urt>>::Hold,
        ) -> &'b mut <Self as RegRef<'a, Urt>>::Hold,
    {
        unsafe {
            write_volatile(self.as_mut_ptr(), f(&mut self.default()).val().bits());
        }
    }

    #[inline]
    fn store_val(&mut self, val: Self::Val) {
        self.store_bits(val.bits());
    }

    #[inline]
    fn store_bits(&mut self, bits: <Self::Val as Bitfield>::Bits) {
        unsafe { write_volatile(self.as_mut_ptr(), bits) };
    }

    #[inline]
    fn reset(&'a mut self) {
        unsafe { write_volatile(self.as_mut_ptr(), self.default_val().bits()) };
    }
}

impl<'a, T, R> WRegAtomic<'a, T> for R
where
    T: RegAtomic,
    R: WReg<T> + RegRef<'a, T>,
    // Extra bound to make the dot operator checking `WRegUnsync` first.
    R::Val: Bitfield,
{
    #[inline]
    fn store<F>(&'a self, f: F)
    where
        F: for<'b> FnOnce(
            &'b mut <Self as RegRef<'a, T>>::Hold,
        ) -> &'b mut <Self as RegRef<'a, T>>::Hold,
    {
        self.store_val(f(&mut self.default()).val());
    }

    #[inline]
    fn store_val(&self, val: Self::Val) {
        self.store_bits(val.bits());
    }

    #[inline]
    fn store_bits(&self, bits: <Self::Val as Bitfield>::Bits) {
        unsafe { write_volatile(self.as_mut_ptr(), bits) };
    }

    #[inline]
    fn reset(&'a self) {
        self.store_val(self.default_val());
    }
}

impl<'a, R> RwRegUnsync<'a> for R
where
    R: RReg<Urt> + WRegUnsync<'a> + RegRef<'a, Urt>,
{
    #[inline]
    fn modify<F>(&'a mut self, f: F)
    where
        F: for<'b> FnOnce(
            &'b mut <Self as RegRef<'a, Urt>>::Hold,
        ) -> &'b mut <Self as RegRef<'a, Urt>>::Hold,
    {
        unsafe {
            write_volatile(self.as_mut_ptr(), f(&mut self.load()).val().bits());
        }
    }
}

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
    //!
    //! ```compile_fail
    //! #![feature(proc_macro_hygiene)]
    //! use drone_core::reg;
    //! reg::tokens!(macro reg_tokens; crate; crate;);
    //! reg_tokens!(struct Regs1;);
    //! reg_tokens!(struct Regs2;);
    //! ```
    //!
    //! ```compile_fail
    //! #![feature(proc_macro_hygiene)]
    //! use drone_core::reg;
    //! reg::tokens!(macro reg_tokens1; crate; crate;);
    //! reg::tokens!(macro reg_tokens2; crate; crate;);
    //! reg_tokens1!(struct Regs1;);
    //! reg_tokens2!(struct Regs2;);
    //! ```
}
