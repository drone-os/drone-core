//! Peripheral is a group of [`memory-mapped registers`](crate::reg) or their
//! fields.
//!
//! # Singular Peripheral
//!
//! Singular peripheral is a unique group of registers or their fields that have
//! a common purpose. Here is an example of how to define and use it:
//!
//! ```
//! # #![feature(proc_macro_hygiene)]
//! # use drone_core::reg;
//! # use drone_core::reg::prelude::*;
//! # reg!(pub mod RCC APB1ENR1; 0 32 0; RTCAPBEN { 0 1 });
//! # reg!(pub mod RTC TR; 0 32 0;);
//! # reg!(pub mod RTC DR; 0 32 0;);
//! # reg!(pub mod RTC CR; 0 32 0;);
//! # reg::tokens! {
//! #     macro reg_tokens; crate; crate;
//! #     mod RCC { APB1ENR1; }
//! #     mod RTC { TR; DR; CR; }
//! # }
//! # reg_tokens!(index => Regs;);
//! use core::mem::size_of_val;
//! use drone_core::periph;
//!
//! periph::singular! {
//!     /// Extracts RTC register tokens.
//!     pub macro periph_rtc;
//!     /// Real-Time Clock peripheral.
//!     pub struct RtcPeriph;
//!
//!     // Path prefix to reach registers.
//!     crate;
//!     // Absolute path to the current module.
//!     crate;
//!
//!     // In the register block RCC...
//!     RCC {
//!         // In the register APB1ENR1...
//!         APB1ENR1 {
//!             // Map the single field RTCAPBEN. Other fields in this register
//!             // could be used by other peripherals.
//!             RTCAPBEN;
//!         }
//!     }
//!     // In the register block RTC...
//!     RTC {
//!         // Map the whole registers TR, DR, and CR.
//!         TR;
//!         DR;
//!         CR;
//!     }
//! }
//!
//! // This will expand to the struct and the macro below:
//!
//! # mod _scope {
//! #     use super::*;
//! /// Real-Time Clock.
//! pub struct RtcPeriph {
//!     pub rcc_apb1enr1_rtcapben: rcc::apb1enr1::Rtcapben<Srt>,
//!     pub rtc_tr: rtc::Tr<Srt>,
//!     pub rtc_dr: rtc::Dr<Srt>,
//!     pub rtc_cr: rtc::Cr<Srt>,
//! }
//!
//! /// Extracts RTC register tokens.
//! macro_rules! periph_rtc {
//!     ($reg:ident) => {
//!         RtcPeriph {
//!             rcc_apb1enr1_rtcapben: $reg.rcc_apb1enr1.rtcapben,
//!             rtc_tr: $reg.rtc_tr,
//!             rtc_dr: $reg.rtc_dr,
//!             rtc_cr: $reg.rtc_cr,
//!         }
//!     };
//! }
//! # }
//!
//! // Here is how to use it in your `trunk` thread:
//!
//! fn trunk(reg: Regs) {
//!     let rtc = periph_rtc!(reg);
//!     assert_eq!(size_of_val(&rtc), 0);
//! }
//!
//! # fn main() { trunk(unsafe { drone_core::token::Token::take()} ); }
//! ```
//!
//! # Generic Peripheral
//!
//! Often there are multiple peripherals of a single type. For example in
//! STM32L4S9 microcontroller there are USART1, USART2, USART3, UART4, UART5,
//! and LPUART1. Most of their registers are the same, but also there are some
//! differences. USART1, USART2, USART3 support synchronous transmission, and
//! LPUART1 can function in low-power modes. However their drivers would have
//! many functions in common. For this reason we map those peripheral registers
//! together in a generic structure, and also map their differences. Here is an
//! example:
//!
//! ```
//! # #![feature(proc_macro_hygiene)]
//! # use drone_core::reg;
//! # use drone_core::reg::prelude::*;
//! # reg!(pub mod RCC APB1ENR1; 0 32 0 RReg WReg; UART4EN { 0 1 RRRegField WWRegField }
//! #                                              UARTRST { 0 1 RRRegField WWRegField });
//! # reg!(pub mod UART4 CR1; 0 32 0 RReg WReg; CMIE { 0 1 RRRegField WWRegField });
//! # reg!(pub mod UART4 RTOR; 0 32 0 RReg WReg; BLEN { 0 2 RRRegField WWRegField });
//! # reg::tokens! {
//! #     macro reg_tokens; crate; crate;
//! #     mod RCC { APB1ENR1; }
//! #     mod UART4 { CR1; RTOR; }
//! # }
//! # reg_tokens!(index => Regs;);
//! # fn main() {}
//! use drone_core::{periph, reg::marker::*};
//!
//! // Here we define the generic UART peripheral.
//! periph! {
//!     /// Generic Universal Asynchronous Receiver/Transmitter peripheral variant.
//!     pub trait UartMap {
//!         // Concrete UART peripherals will implement this trait. Arbitrary code
//!         // can be placed here.
//!     }
//!     // This will be the peripheral struct with public fields corresponding to
//!     // registers and/or register fields. The signature is
//!     // `struct UartPeriph<T: UartMap>`.
//!     /// Generic Universal Asynchronous Receiver/Transmitter peripheral.
//!     pub struct UartPeriph;
//!
//!     // With RCC namespace...
//!     RCC {
//!         APBENR {
//!             // We need to declare the size of the register and its properties.
//!             // `RwReg` is a marker trait from `drone_core::reg::marker`, and it
//!             // means this is a read-write register. `Shared` is a special
//!             // property, which means the peripheral will not own the whole
//!             // register, but will own only a part of its fields.
//!             0x20 RwReg Shared;
//!             // All peripherals will have UARTEN field. Again, `RwRwRegFieldBit`
//!             // is a marker trait from `drone_core::reg::marker`, and it means
//!             // this is a read-write single-bit field.
//!             UARTEN { RwRwRegFieldBit }
//!             // This is an optional field. Not all concrete peripherals will have
//!             // it.
//!             UARTRST { RwRwRegFieldBit Option }
//!         }
//!     }
//!     // Actually there is no UART register block. There are USART1, USART2,
//!     // USART3 and so on. This namespace is virtual; concrete peripherals
//!     // will map actual blocks to this namespace.
//!     UART {
//!         CR1 {
//!             0x20 RwReg;
//!             CMIE { RwRwRegFieldBit }
//!             EOBIE { RwRwRegFieldBit Option }
//!         }
//!         RTOR {
//!             // This is an optional register.
//!             0x20 RwReg Option;
//!             BLEN { RwRwRegFieldBits }
//!             // And this is an optional field of the optional register.
//!             RTO { RwRwRegFieldBits Option }
//!         }
//!     }
//! }
//!
//! // Here we define the concrete UART4 peripheral.
//! periph::map! {
//!     // Extracts UART4 register tokens.
//!     pub macro periph_uart4;
//!     // UART4 peripheral variant.
//!     pub struct Uart4;
//!
//!     impl UartMap for Uart4 {
//!         // If `UartMap` defined some items, they should be implemented here.
//!     }
//!
//!     // Path prefix to reach registers.
//!     crate;
//!     // Absolute path to the current module.
//!     crate;
//!
//!     RCC {
//!         APBENR {
//!             // Here we provide the real name of the register - APB1ENR1. And
//!             // also the special properties like `Shared` or `Option`.
//!             APB1ENR1 Shared;
//!             // Again, we provide the real name of the field.
//!             UARTEN { UART4EN }
//!             // If the name is the same, we should provide it. Also if an
//!             // optional field present, we should mark it with `Option`.
//!             UARTRST { UARTRST Option }
//!         }
//!     }
//!     UART {
//!         // The real name of the block of registers.
//!         UART4;
//!         CR1 {
//!             CR1;
//!             CMIE { CMIE }
//!             // If the optional field absent, we should mention it like this.
//!             EOBIE {}
//!         }
//!         RTOR {
//!             RTOR Option;
//!             BLEN { BLEN }
//!             RTO {}
//!         }
//!     }
//! }
//!
//! // Here is how we define a function generic over all variants of the peripheral.
//! // Optional fields will not be available even if the concrete peripheral has them.
//! fn basic_fields<T: UartMap>(uart: UartPeriph<T>) {}
//!
//! // Here is a generic function over peripherals that have all optional fields.
//! fn opt_fields<T>(uart: UartPeriph<T>)
//! where
//!     T: UartMap + RccApbenrUartrst + UartCr1Eobie + UartRtorRto,
//! {
//! }
//! ```

/// Implements the generic peripheral.
///
/// See [the module level documentation](self) for details.
#[doc(inline)]
pub use drone_core_macros::periph_map as map;

/// Defines a singular peripheral.
///
/// See [the module level documentation](self) for details.
#[doc(inline)]
pub use drone_core_macros::periph_singular as singular;
