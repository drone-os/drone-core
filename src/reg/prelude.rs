//! The Memory-Mapped Registers prelude.
//!
//! The purpose of this module is to alleviate imports of many common `reg`
//! traits by adding a glob import to the top of `reg` heavy modules:
//!
//! ```
//! # #![allow(unused_imports)]
//! use drone_core::reg::prelude::*;
//! ```

#[doc(no_inline)]
pub use crate::reg::{
    field::{RRRegField, RegField, RoRRegField, WWRegField, WoWRegField},
    tag::{Crt, RegAtomic, RegOwned, RegTag, Srt, Urt},
    RReg, Reg, RegHold, RoReg, WReg, WoReg,
};
#[doc(no_inline)]
pub use crate::reg::{
    field::{
        RRRegFieldBit as _, RRRegFieldBits as _, RegFieldBit as _, RegFieldBits as _,
        WWRegFieldBit as _, WWRegFieldBits as _, WoWoRegField as _, WoWoRegFieldBit as _,
        WoWoRegFieldBits as _,
    },
    RwRegUnsync as _, WRegAtomic as _, WRegUnsync as _,
};
#[cfg(feature = "atomics")]
#[doc(no_inline)]
pub use crate::reg::{
    field::{WRwRegFieldAtomic as _, WRwRegFieldBitAtomic as _, WRwRegFieldBitsAtomic as _},
    RwRegAtomic as _,
};
#[cfg(not(feature = "atomics"))]
#[doc(no_inline)]
pub use crate::reg::{
    field::{
        WRwRegFieldBitSoftAtomic as _, WRwRegFieldBitsSoftAtomic as _, WRwRegFieldSoftAtomic as _,
    },
    RwRegSoftAtomic as _,
};
