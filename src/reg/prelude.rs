//! The Memory-Mapped Registers prelude.
//!
//! The purpose of this module is to alleviate imports of many common `reg`
//! traits by adding a glob import to the top of `reg` heavy modules:
//!
//! ```
//! # #![allow(unused_imports)]
//! use drone_core::reg::prelude::*;
//! ```

pub use crate::reg::{
    field::{RRRegField, RegField, RoRRegField, WWRegField, WoWRegField},
    tag::{Crt, RegAtomic, RegOwned, RegTag, Srt, Urt},
    RReg, Reg, RegHold, RoReg, WReg, WoReg,
};

pub use crate::reg::{
    field::{
        RRRegFieldBit as _, RRRegFieldBits as _, RegFieldBit as _, RegFieldBits as _,
        WWRegFieldBit as _, WWRegFieldBits as _, WoWoRegField as _, WoWoRegFieldBit as _,
        WoWoRegFieldBits as _,
    },
    RegRef as _, RwRegUnsync as _, WRegAtomic as _, WRegUnsync as _,
};
