//! Memory-mapped registers prelude.

pub use super::{
  Crt, RRRegField, RReg, Reg, RegAtomic, RegField, RegHold, RegOwned, RegTag,
  RoRRegField, RoReg, Srt, Urt, WReg, WWRegField, WoReg, WoWRegField,
};

pub use super::{
  RRRegFieldBit as _, RRRegFieldBits as _, RegFieldBit as _, RegFieldBits as _,
  RegRef as _, RwRegUnsync as _, WRegAtomic as _, WRegUnsync as _,
  WWRegFieldBit as _, WWRegFieldBits as _, WoWoRegField as _,
  WoWoRegFieldBit as _, WoWoRegFieldBits as _,
};
