//! Marker traits representing properties of memory-mapped registers.

#[doc(inline)]
pub use crate::reg::{
    field::{WoWoRegFieldBit, WoWoRegFieldBits},
    RoReg, WoReg,
};

use crate::reg::{
    field::{
        RRRegFieldBit, RRRegFieldBits, RoRRegField, WWRegFieldBit, WWRegFieldBits, WoWRegField,
    },
    tag::{Crt, RegTag, Srt, Urt},
    RReg, RwRegUnsync, WReg, WRegAtomic, WRegUnsync,
};

/// Read-write register.
#[marker]
pub trait RwReg<T: RegTag>
where
    Self: RReg<T>,
    Self: WReg<T>,
{
}

impl<R, T: RegTag> RwReg<T> for R
where
    R: RReg<T>,
    R: WReg<T>,
{
}

/// Unsynchronized read-write register.
#[marker]
pub trait URwReg
where
    Self: RwReg<Urt>,
    Self: for<'a> RwRegUnsync<'a>,
{
}

impl<R> URwReg for R
where
    R: RwReg<Urt>,
    R: for<'a> RwRegUnsync<'a>,
{
}

/// Unsynchronized read-only register.
#[marker]
pub trait URoReg
where
    Self: RoReg<Urt>,
{
}

impl<R> URoReg for R where R: RoReg<Urt> {}

/// Unsynchronized write-only register.
#[marker]
pub trait UWoReg
where
    Self: WoReg<Urt>,
    Self: for<'a> WRegUnsync<'a>,
{
}

impl<R> UWoReg for R
where
    R: WoReg<Urt>,
    R: for<'a> WRegUnsync<'a>,
{
}

/// Synchronized read-write register.
#[marker]
pub trait SRwReg
where
    Self: RwReg<Srt>,
    Self: for<'a> WRegAtomic<'a, Srt>,
{
}

impl<R> SRwReg for R
where
    R: RwReg<Srt>,
    R: for<'a> WRegAtomic<'a, Srt>,
{
}

/// Synchronized read-only register.
#[marker]
pub trait SRoReg
where
    Self: RoReg<Srt>,
{
}

impl<R> SRoReg for R where R: RoReg<Srt> {}

/// Synchronized write-only register.
#[marker]
pub trait SWoReg
where
    Self: WoReg<Srt>,
    Self: for<'a> WRegAtomic<'a, Srt>,
{
}

impl<R> SWoReg for R
where
    R: WoReg<Srt>,
    R: for<'a> WRegAtomic<'a, Srt>,
{
}

/// Copyable read-write register.
#[marker]
pub trait CRwReg
where
    Self: RwReg<Crt>,
    Self: for<'a> WRegAtomic<'a, Crt>,
    Self: Copy,
{
}

impl<R> CRwReg for R
where
    R: RwReg<Crt>,
    R: for<'a> WRegAtomic<'a, Crt>,
    R: Copy,
{
}

/// Copyable read-only register.
#[marker]
pub trait CRoReg
where
    Self: RoReg<Crt>,
    Self: Copy,
{
}

impl<R> CRoReg for R
where
    R: RoReg<Crt>,
    R: Copy,
{
}

/// Copyable write-only register.
#[marker]
pub trait CWoReg
where
    Self: WoReg<Crt>,
    Self: for<'a> WRegAtomic<'a, Crt>,
    Self: Copy,
{
}

impl<R> CWoReg for R
where
    R: WoReg<Crt>,
    R: for<'a> WRegAtomic<'a, Crt>,
    R: Copy,
{
}

/// Single-bit read-write field of read-write register.
#[marker]
pub trait RwRwRegFieldBit<T: RegTag>
where
    Self: WWRegFieldBit<T>,
    Self: RRRegFieldBit<T>,
    Self::Reg: RwReg<T>,
{
}

impl<R, T: RegTag> RwRwRegFieldBit<T> for R
where
    R: WWRegFieldBit<T>,
    R: RRRegFieldBit<T>,
    R::Reg: RwReg<T>,
{
}

/// Multi-bit read-write field of read-write register.
#[marker]
pub trait RwRwRegFieldBits<T: RegTag>
where
    Self: WWRegFieldBits<T>,
    Self: RRRegFieldBits<T>,
    Self::Reg: RwReg<T>,
{
}

impl<R, T: RegTag> RwRwRegFieldBits<T> for R
where
    R: WWRegFieldBits<T>,
    R: RRRegFieldBits<T>,
    R::Reg: RwReg<T>,
{
}

/// Single-bit write-only field of read-write register.
#[marker]
pub trait WoRwRegFieldBit<T: RegTag>
where
    Self: WWRegFieldBit<T>,
    Self: WoWRegField<T>,
    Self::Reg: RwReg<T>,
{
}

impl<R, T: RegTag> WoRwRegFieldBit<T> for R
where
    R: WWRegFieldBit<T>,
    R: WoWRegField<T>,
    R::Reg: RwReg<T>,
{
}

/// Multi-bit write-only field of read-write register.
#[marker]
pub trait WoRwRegFieldBits<T: RegTag>
where
    Self: WWRegFieldBits<T>,
    Self: WoWRegField<T>,
    Self::Reg: RwReg<T>,
{
}

impl<R, T: RegTag> WoRwRegFieldBits<T> for R
where
    R: WWRegFieldBits<T>,
    R: WoWRegField<T>,
    R::Reg: RwReg<T>,
{
}

/// Single-bit read-only field of read-write register.
#[marker]
pub trait RoRwRegFieldBit<T: RegTag>
where
    Self: RRRegFieldBit<T>,
    Self: RoRRegField<T>,
    Self::Reg: RwReg<T>,
{
}

impl<R, T: RegTag> RoRwRegFieldBit<T> for R
where
    R: RRRegFieldBit<T>,
    R: RoRRegField<T>,
    R::Reg: RwReg<T>,
{
}

/// Multi-bit read-only field of read-write register.
#[marker]
pub trait RoRwRegFieldBits<T: RegTag>
where
    Self: RRRegFieldBits<T>,
    Self: RoRRegField<T>,
    Self::Reg: RwReg<T>,
{
}

impl<R, T: RegTag> RoRwRegFieldBits<T> for R
where
    R: RRRegFieldBits<T>,
    R: RoRRegField<T>,
    R::Reg: RwReg<T>,
{
}

/// Single-bit read-only field of read-only register.
#[marker]
pub trait RoRoRegFieldBit<T: RegTag>
where
    Self: RRRegFieldBit<T>,
    Self: RoRRegField<T>,
    Self::Reg: RoReg<T>,
{
}

impl<R, T: RegTag> RoRoRegFieldBit<T> for R
where
    R: RRRegFieldBit<T>,
    R: RoRRegField<T>,
    R::Reg: RoReg<T>,
{
}

/// Multi-bit read-only field of read-only register.
#[marker]
pub trait RoRoRegFieldBits<T: RegTag>
where
    Self: RRRegFieldBits<T>,
    Self: RoRRegField<T>,
    Self::Reg: RoReg<T>,
{
}

impl<R, T: RegTag> RoRoRegFieldBits<T> for R
where
    R: RRRegFieldBits<T>,
    R: RoRRegField<T>,
    R::Reg: RoReg<T>,
{
}

/// Unsynchronized single-bit read-write field of read-write register.
#[marker]
pub trait URwRwRegFieldBit
where
    Self: RwRwRegFieldBit<Urt>,
    Self::Reg: URwReg,
{
}

impl<R> URwRwRegFieldBit for R
where
    R: RwRwRegFieldBit<Urt>,
    R::Reg: URwReg,
{
}

/// Unsynchronized multi-bit read-write field of read-write register.
#[marker]
pub trait URwRwRegFieldBits
where
    Self: RwRwRegFieldBits<Urt>,
    Self::Reg: URwReg,
{
}

impl<R> URwRwRegFieldBits for R
where
    R: RwRwRegFieldBits<Urt>,
    R::Reg: URwReg,
{
}

/// Unsynchronized single-bit write-only field of read-write register.
#[marker]
pub trait UWoRwRegFieldBit
where
    Self: WoRwRegFieldBit<Urt>,
    Self::Reg: URwReg,
{
}

impl<R> UWoRwRegFieldBit for R
where
    R: WoRwRegFieldBit<Urt>,
    R::Reg: URwReg,
{
}

/// Unsynchronized multi-bit write-only field of read-write register.
#[marker]
pub trait UWoRwRegFieldBits
where
    Self: WoRwRegFieldBits<Urt>,
    Self::Reg: URwReg,
{
}

impl<R> UWoRwRegFieldBits for R
where
    R: WoRwRegFieldBits<Urt>,
    R::Reg: URwReg,
{
}

/// Unsynchronized single-bit write-only field of write-only register.
#[marker]
pub trait UWoWoRegFieldBit
where
    Self: WoWoRegFieldBit<Urt>,
    Self::Reg: UWoReg,
{
}

impl<R> UWoWoRegFieldBit for R
where
    R: WoWoRegFieldBit<Urt>,
    R::Reg: UWoReg,
{
}

/// Unsynchronized multi-bit write-only field of write-only register.
#[marker]
pub trait UWoWoRegFieldBits
where
    Self: WoWoRegFieldBits<Urt>,
    Self::Reg: UWoReg,
{
}

impl<R> UWoWoRegFieldBits for R
where
    R: WoWoRegFieldBits<Urt>,
    R::Reg: UWoReg,
{
}

/// Unsynchronized single-bit read-only field of read-write register.
#[marker]
pub trait URoRwRegFieldBit
where
    Self: RoRwRegFieldBit<Urt>,
    Self::Reg: URwReg,
{
}

impl<R> URoRwRegFieldBit for R
where
    R: RoRwRegFieldBit<Urt>,
    R::Reg: URwReg,
{
}

/// Unsynchronized multi-bit read-only field of read-write register.
#[marker]
pub trait URoRwRegFieldBits
where
    Self: RoRwRegFieldBits<Urt>,
    Self::Reg: URwReg,
{
}

impl<R> URoRwRegFieldBits for R
where
    R: RoRwRegFieldBits<Urt>,
    R::Reg: URwReg,
{
}

/// Unsynchronized single-bit read-only field of read-only register.
#[marker]
pub trait URoRoRegFieldBit
where
    Self: RoRoRegFieldBit<Urt>,
    Self::Reg: URoReg,
{
}

impl<R> URoRoRegFieldBit for R
where
    R: RoRoRegFieldBit<Urt>,
    R::Reg: URoReg,
{
}

/// Unsynchronized multi-bit read-only field of read-only register.
#[marker]
pub trait URoRoRegFieldBits
where
    Self: RoRoRegFieldBits<Urt>,
    Self::Reg: URoReg,
{
}

impl<R> URoRoRegFieldBits for R
where
    R: RoRoRegFieldBits<Urt>,
    R::Reg: URoReg,
{
}

/// Synchronized single-bit read-write field of read-write register.
#[marker]
pub trait SRwRwRegFieldBit
where
    Self: RwRwRegFieldBit<Srt>,
    Self::Reg: SRwReg,
{
}

impl<R> SRwRwRegFieldBit for R
where
    R: RwRwRegFieldBit<Srt>,
    R::Reg: SRwReg,
{
}

/// Synchronized multi-bit read-write field of read-write register.
#[marker]
pub trait SRwRwRegFieldBits
where
    Self: RwRwRegFieldBits<Srt>,
    Self::Reg: SRwReg,
{
}

impl<R> SRwRwRegFieldBits for R
where
    R: RwRwRegFieldBits<Srt>,
    R::Reg: SRwReg,
{
}

/// Synchronized single-bit write-only field of read-write register.
#[marker]
pub trait SWoRwRegFieldBit
where
    Self: WoRwRegFieldBit<Srt>,
    Self::Reg: SRwReg,
{
}

impl<R> SWoRwRegFieldBit for R
where
    R: WoRwRegFieldBit<Srt>,
    R::Reg: SRwReg,
{
}

/// Synchronized multi-bit write-only field of read-write register.
#[marker]
pub trait SWoRwRegFieldBits
where
    Self: WoRwRegFieldBits<Srt>,
    Self::Reg: SRwReg,
{
}

impl<R> SWoRwRegFieldBits for R
where
    R: WoRwRegFieldBits<Srt>,
    R::Reg: SRwReg,
{
}

/// Synchronized single-bit write-only field of write-only register.
#[marker]
pub trait SWoWoRegFieldBit
where
    Self: WoWoRegFieldBit<Srt>,
    Self::Reg: SWoReg,
{
}

impl<R> SWoWoRegFieldBit for R
where
    R: WoWoRegFieldBit<Srt>,
    R::Reg: SWoReg,
{
}

/// Synchronized multi-bit write-only field of write-only register.
#[marker]
pub trait SWoWoRegFieldBits
where
    Self: WoWoRegFieldBits<Srt>,
    Self::Reg: SWoReg,
{
}

impl<R> SWoWoRegFieldBits for R
where
    R: WoWoRegFieldBits<Srt>,
    R::Reg: SWoReg,
{
}

/// Synchronized single-bit read-only field of read-write register.
#[marker]
pub trait SRoRwRegFieldBit
where
    Self: RoRwRegFieldBit<Srt>,
    Self::Reg: SRwReg,
{
}

impl<R> SRoRwRegFieldBit for R
where
    R: RoRwRegFieldBit<Srt>,
    R::Reg: SRwReg,
{
}

/// Synchronized multi-bit read-only field of read-write register.
#[marker]
pub trait SRoRwRegFieldBits
where
    Self: RoRwRegFieldBits<Srt>,
    Self::Reg: SRwReg,
{
}

impl<R> SRoRwRegFieldBits for R
where
    R: RoRwRegFieldBits<Srt>,
    R::Reg: SRwReg,
{
}

/// Synchronized single-bit read-only field of read-only register.
#[marker]
pub trait SRoRoRegFieldBit
where
    Self: RoRoRegFieldBit<Srt>,
    Self::Reg: SRoReg,
{
}

impl<R> SRoRoRegFieldBit for R
where
    R: RoRoRegFieldBit<Srt>,
    R::Reg: SRoReg,
{
}

/// Synchronized multi-bit read-only field of read-only register.
#[marker]
pub trait SRoRoRegFieldBits
where
    Self: RoRoRegFieldBits<Srt>,
    Self::Reg: SRoReg,
{
}

impl<R> SRoRoRegFieldBits for R
where
    R: RoRoRegFieldBits<Srt>,
    R::Reg: SRoReg,
{
}

/// Copyable single-bit read-write field of read-write register.
#[marker]
pub trait CRwRwRegFieldBit
where
    Self: RwRwRegFieldBit<Crt>,
    Self: Copy,
    Self::Reg: CRwReg,
{
}

impl<R> CRwRwRegFieldBit for R
where
    R: RwRwRegFieldBit<Crt>,
    R: Copy,
    R::Reg: CRwReg,
{
}

/// Copyable multi-bit read-write field of read-write register.
#[marker]
pub trait CRwRwRegFieldBits
where
    Self: RwRwRegFieldBits<Crt>,
    Self: Copy,
    Self::Reg: CRwReg,
{
}

impl<R> CRwRwRegFieldBits for R
where
    R: RwRwRegFieldBits<Crt>,
    R: Copy,
    R::Reg: CRwReg,
{
}

/// Copyable single-bit write-only field of read-write register.
#[marker]
pub trait CWoRwRegFieldBit
where
    Self: WoRwRegFieldBit<Crt>,
    Self: Copy,
    Self::Reg: CRwReg,
{
}

impl<R> CWoRwRegFieldBit for R
where
    R: WoRwRegFieldBit<Crt>,
    R: Copy,
    R::Reg: CRwReg,
{
}

/// Copyable multi-bit write-only field of read-write register.
#[marker]
pub trait CWoRwRegFieldBits
where
    Self: WoRwRegFieldBits<Crt>,
    Self: Copy,
    Self::Reg: CRwReg,
{
}

impl<R> CWoRwRegFieldBits for R
where
    R: WoRwRegFieldBits<Crt>,
    R: Copy,
    R::Reg: CRwReg,
{
}

/// Copyable single-bit write-only field of write-only register.
#[marker]
pub trait CWoWoRegFieldBit
where
    Self: WoWoRegFieldBit<Crt>,
    Self: Copy,
    Self::Reg: CWoReg,
{
}

impl<R> CWoWoRegFieldBit for R
where
    R: WoWoRegFieldBit<Crt>,
    R: Copy,
    R::Reg: CWoReg,
{
}

/// Copyable multi-bit write-only field of write-only register.
#[marker]
pub trait CWoWoRegFieldBits
where
    Self: WoWoRegFieldBits<Crt>,
    Self: Copy,
    Self::Reg: CWoReg,
{
}

impl<R> CWoWoRegFieldBits for R
where
    R: WoWoRegFieldBits<Crt>,
    R: Copy,
    R::Reg: CWoReg,
{
}

/// Copyable single-bit read-only field of read-write register.
#[marker]
pub trait CRoRwRegFieldBit
where
    Self: RoRwRegFieldBit<Crt>,
    Self: Copy,
    Self::Reg: CRwReg,
{
}

impl<R> CRoRwRegFieldBit for R
where
    R: RoRwRegFieldBit<Crt>,
    R: Copy,
    R::Reg: CRwReg,
{
}

/// Copyable multi-bit read-only field of read-write register.
#[marker]
pub trait CRoRwRegFieldBits
where
    Self: RoRwRegFieldBits<Crt>,
    Self: Copy,
    Self::Reg: CRwReg,
{
}

impl<R> CRoRwRegFieldBits for R
where
    R: RoRwRegFieldBits<Crt>,
    R: Copy,
    R::Reg: CRwReg,
{
}

/// Copyable single-bit read-only field of read-only register.
#[marker]
pub trait CRoRoRegFieldBit
where
    Self: RoRoRegFieldBit<Crt>,
    Self: Copy,
    Self::Reg: CRoReg,
{
}

impl<R> CRoRoRegFieldBit for R
where
    R: RoRoRegFieldBit<Crt>,
    R: Copy,
    R::Reg: CRoReg,
{
}

/// Copyable multi-bit read-only field of read-only register.
#[marker]
pub trait CRoRoRegFieldBits
where
    Self: RoRoRegFieldBits<Crt>,
    Self: Copy,
    Self::Reg: CRoReg,
{
}

impl<R> CRoRoRegFieldBits for R
where
    R: RoRoRegFieldBits<Crt>,
    R: Copy,
    R::Reg: CRoReg,
{
}
