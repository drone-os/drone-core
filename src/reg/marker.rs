//! Marker traits for memory-mapped registers.

use reg::prelude::*;

// {{{ URwReg
/// Unsynchronized read-write register token.
pub trait URwReg
where
  Self: RReg<Urt>,
  Self: WReg<Urt>,
  Self: for<'a> RwRegUnsync<'a>,
{
}

impl<R> URwReg for R
where
  R: RReg<Urt>,
  R: WReg<Urt>,
  R: for<'a> RwRegUnsync<'a>,
{
}

// }}}
// {{{ URoReg
/// Unsynchronized read-only register token.
pub trait URoReg
where
  Self: RoReg<Urt>,
{
}

impl<R> URoReg for R
where
  R: RoReg<Urt>,
{
}

// }}}
// {{{ UWoReg
/// Unsynchronized write-only register token.
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

// }}}
// {{{ SRwReg
/// Synchronized read-write register token.
pub trait SRwReg
where
  Self: RReg<Srt>,
  Self: WReg<Srt>,
{
}

impl<R> SRwReg for R
where
  R: RReg<Srt>,
  R: WReg<Srt>,
{
}

// }}}
// {{{ SRoReg
/// Synchronized read-only register token.
pub trait SRoReg
where
  Self: RoReg<Srt>,
{
}

impl<R> SRoReg for R
where
  R: RoReg<Srt>,
{
}

// }}}
// {{{ SWoReg
/// Synchronized write-only register token.
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

// }}}
// {{{ FRwReg
/// Forkable read-write register token.
pub trait FRwReg
where
  Self: RReg<Frt>,
  Self: WReg<Frt>,
  Self: RegFork,
{
}

impl<R> FRwReg for R
where
  R: RReg<Frt>,
  R: WReg<Frt>,
  R: RegFork,
{
}

// }}}
// {{{ FRoReg
/// Forkable read-only register token.
pub trait FRoReg
where
  Self: RoReg<Frt>,
  Self: RegFork,
{
}

impl<R> FRoReg for R
where
  R: RoReg<Frt>,
  R: RegFork,
{
}

// }}}
// {{{ FWoReg
/// Forkable write-only register token.
pub trait FWoReg
where
  Self: WoReg<Frt>,
  Self: for<'a> WRegAtomic<'a, Frt>,
  Self: RegFork,
{
}

impl<R> FWoReg for R
where
  R: WoReg<Frt>,
  R: for<'a> WRegAtomic<'a, Frt>,
  R: RegFork,
{
}

// }}}
// {{{ URwRwRegFieldBit
/// Unsynchronized one-bit read-write field of read-write register token.
pub trait URwRwRegFieldBit
where
  Self: WWRegFieldBit<Urt>,
  Self: RRRegFieldBit<Urt>,
  Self::Reg: URwReg,
{
}

impl<R> URwRwRegFieldBit for R
where
  R: WWRegFieldBit<Urt>,
  R: RRRegFieldBit<Urt>,
  R::Reg: URwReg,
{
}

// }}}
// {{{ URwRwRegFieldBits
/// Unsynchronized multi-bit read-write field of read-write register token.
pub trait URwRwRegFieldBits
where
  Self: WWRegFieldBits<Urt>,
  Self: RRRegFieldBits<Urt>,
  Self::Reg: URwReg,
{
}

impl<R> URwRwRegFieldBits for R
where
  R: WWRegFieldBits<Urt>,
  R: RRRegFieldBits<Urt>,
  R::Reg: URwReg,
{
}

// }}}
// {{{ UWoRwRegFieldBit
/// Unsynchronized one-bit write-only field of read-write register token.
pub trait UWoRwRegFieldBit
where
  Self: WWRegFieldBit<Urt>,
  Self: WoWRegField<Urt>,
  Self::Reg: URwReg,
{
}

impl<R> UWoRwRegFieldBit for R
where
  R: WWRegFieldBit<Urt>,
  R: WoWRegField<Urt>,
  R::Reg: URwReg,
{
}

// }}}
// {{{ UWoRwRegFieldBits
/// Unsynchronized multi-bit write-only field of read-write register token.
pub trait UWoRwRegFieldBits
where
  Self: WWRegFieldBits<Urt>,
  Self: WoWRegField<Urt>,
  Self::Reg: URwReg,
{
}

impl<R> UWoRwRegFieldBits for R
where
  R: WWRegFieldBits<Urt>,
  R: WoWRegField<Urt>,
  R::Reg: URwReg,
{
}

// }}}
// {{{ UWoWoRegFieldBit
/// Unsynchronized one-bit write-only field of write-only register token.
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

// }}}
// {{{ UWoWoRegFieldBits
/// Unsynchronized multi-bit write-only field of write-only register token.
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

// }}}
// {{{ URoRwRegFieldBit
/// Unsynchronized one-bit read-only field of read-write register token.
pub trait URoRwRegFieldBit
where
  Self: RRRegFieldBit<Urt>,
  Self: RoRRegField<Urt>,
  Self::Reg: URwReg,
{
}

impl<R> URoRwRegFieldBit for R
where
  R: RRRegFieldBit<Urt>,
  R: RoRRegField<Urt>,
  R::Reg: URwReg,
{
}

// }}}
// {{{ URoRwRegFieldBits
/// Unsynchronized multi-bit read-only field of read-write register token.
pub trait URoRwRegFieldBits
where
  Self: RRRegFieldBits<Urt>,
  Self: RoRRegField<Urt>,
  Self::Reg: URwReg,
{
}

impl<R> URoRwRegFieldBits for R
where
  R: RRRegFieldBits<Urt>,
  R: RoRRegField<Urt>,
  R::Reg: URwReg,
{
}

// }}}
// {{{ URoRoRegFieldBit
/// Unsynchronized one-bit read-only field of read-only register token.
pub trait URoRoRegFieldBit
where
  Self: RRRegFieldBit<Urt>,
  Self: RoRRegField<Urt>,
  Self::Reg: URoReg,
{
}

impl<R> URoRoRegFieldBit for R
where
  R: RRRegFieldBit<Urt>,
  R: RoRRegField<Urt>,
  R::Reg: URoReg,
{
}

// }}}
// {{{ URoRoRegFieldBits
/// Unsynchronized multi-bit read-only field of read-only register token.
pub trait URoRoRegFieldBits
where
  Self: RRRegFieldBits<Urt>,
  Self: RoRRegField<Urt>,
  Self::Reg: URoReg,
{
}

impl<R> URoRoRegFieldBits for R
where
  R: RRRegFieldBits<Urt>,
  R: RoRRegField<Urt>,
  R::Reg: URoReg,
{
}

// }}}
// {{{ SRwRwRegFieldBit
/// Synchronized one-bit read-write field of read-write register token.
pub trait SRwRwRegFieldBit
where
  Self: WWRegFieldBit<Srt>,
  Self: RRRegFieldBit<Srt>,
  Self::Reg: SRwReg,
{
}

impl<R> SRwRwRegFieldBit for R
where
  R: WWRegFieldBit<Srt>,
  R: RRRegFieldBit<Srt>,
  R::Reg: SRwReg,
{
}

// }}}
// {{{ SRwRwRegFieldBits
/// Synchronized multi-bit read-write field of read-write register token.
pub trait SRwRwRegFieldBits
where
  Self: WWRegFieldBits<Srt>,
  Self: RRRegFieldBits<Srt>,
  Self::Reg: SRwReg,
{
}

impl<R> SRwRwRegFieldBits for R
where
  R: WWRegFieldBits<Srt>,
  R: RRRegFieldBits<Srt>,
  R::Reg: SRwReg,
{
}

// }}}
// {{{ SWoRwRegFieldBit
/// Synchronized one-bit write-only field of read-write register token.
pub trait SWoRwRegFieldBit
where
  Self: WWRegFieldBit<Srt>,
  Self: WoWRegField<Srt>,
  Self::Reg: SRwReg,
{
}

impl<R> SWoRwRegFieldBit for R
where
  R: WWRegFieldBit<Srt>,
  R: WoWRegField<Srt>,
  R::Reg: SRwReg,
{
}

// }}}
// {{{ SWoRwRegFieldBits
/// Synchronized multi-bit write-only field of read-write register token.
pub trait SWoRwRegFieldBits
where
  Self: WWRegFieldBits<Srt>,
  Self: WoWRegField<Srt>,
  Self::Reg: SRwReg,
{
}

impl<R> SWoRwRegFieldBits for R
where
  R: WWRegFieldBits<Srt>,
  R: WoWRegField<Srt>,
  R::Reg: SRwReg,
{
}

// }}}
// {{{ SWoWoRegFieldBit
/// Synchronized one-bit write-only field of write-only register token.
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

// }}}
// {{{ SWoWoRegFieldBits
/// Synchronized multi-bit write-only field of write-only register token.
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

// }}}
// {{{ SRoRwRegFieldBit
/// Synchronized one-bit read-only field of read-write register token.
pub trait SRoRwRegFieldBit
where
  Self: RRRegFieldBit<Srt>,
  Self: RoRRegField<Srt>,
  Self::Reg: SRwReg,
{
}

impl<R> SRoRwRegFieldBit for R
where
  R: RRRegFieldBit<Srt>,
  R: RoRRegField<Srt>,
  R::Reg: SRwReg,
{
}

// }}}
// {{{ SRoRwRegFieldBits
/// Synchronized multi-bit read-only field of read-write register token.
pub trait SRoRwRegFieldBits
where
  Self: RRRegFieldBits<Srt>,
  Self: RoRRegField<Srt>,
  Self::Reg: SRwReg,
{
}

impl<R> SRoRwRegFieldBits for R
where
  R: RRRegFieldBits<Srt>,
  R: RoRRegField<Srt>,
  R::Reg: SRwReg,
{
}

// }}}
// {{{ SRoRoRegFieldBit
/// Synchronized one-bit read-only field of read-only register token.
pub trait SRoRoRegFieldBit
where
  Self: RRRegFieldBit<Srt>,
  Self: RoRRegField<Srt>,
  Self::Reg: SRoReg,
{
}

impl<R> SRoRoRegFieldBit for R
where
  R: RRRegFieldBit<Srt>,
  R: RoRRegField<Srt>,
  R::Reg: SRoReg,
{
}

// }}}
// {{{ SRoRoRegFieldBits
/// Synchronized multi-bit read-only field of read-only register token.
pub trait SRoRoRegFieldBits
where
  Self: RRRegFieldBits<Srt>,
  Self: RoRRegField<Srt>,
  Self::Reg: SRoReg,
{
}

impl<R> SRoRoRegFieldBits for R
where
  R: RRRegFieldBits<Srt>,
  R: RoRRegField<Srt>,
  R::Reg: SRoReg,
{
}

// }}}
// {{{ FRwRwRegFieldBit
/// Forkable one-bit read-write field of read-write register token.
pub trait FRwRwRegFieldBit
where
  Self: WWRegFieldBit<Frt>,
  Self: RRRegFieldBit<Frt>,
  Self: RegFork,
  Self::Reg: FRwReg,
{
}

impl<R> FRwRwRegFieldBit for R
where
  R: WWRegFieldBit<Frt>,
  R: RRRegFieldBit<Frt>,
  R: RegFork,
  R::Reg: FRwReg,
{
}

// }}}
// {{{ FRwRwRegFieldBits
/// Forkable multi-bit read-write field of read-write register token.
pub trait FRwRwRegFieldBits
where
  Self: WWRegFieldBits<Frt>,
  Self: RRRegFieldBits<Frt>,
  Self: RegFork,
  Self::Reg: FRwReg,
{
}

impl<R> FRwRwRegFieldBits for R
where
  R: WWRegFieldBits<Frt>,
  R: RRRegFieldBits<Frt>,
  R: RegFork,
  R::Reg: FRwReg,
{
}

// }}}
// {{{ FWoRwRegFieldBit
/// Forkable one-bit write-only field of read-write register token.
pub trait FWoRwRegFieldBit
where
  Self: WWRegFieldBit<Frt>,
  Self: WoWRegField<Frt>,
  Self: RegFork,
  Self::Reg: FRwReg,
{
}

impl<R> FWoRwRegFieldBit for R
where
  R: WWRegFieldBit<Frt>,
  R: WoWRegField<Frt>,
  R: RegFork,
  R::Reg: FRwReg,
{
}

// }}}
// {{{ FWoRwRegFieldBits
/// Forkable multi-bit write-only field of read-write register token.
pub trait FWoRwRegFieldBits
where
  Self: WWRegFieldBits<Frt>,
  Self: WoWRegField<Frt>,
  Self: RegFork,
  Self::Reg: FRwReg,
{
}

impl<R> FWoRwRegFieldBits for R
where
  R: WWRegFieldBits<Frt>,
  R: WoWRegField<Frt>,
  R: RegFork,
  R::Reg: FRwReg,
{
}

// }}}
// {{{ FWoWoRegFieldBit
/// Forkable one-bit write-only field of write-only register token.
pub trait FWoWoRegFieldBit
where
  Self: WoWoRegFieldBit<Frt>,
  Self: RegFork,
  Self::Reg: FWoReg,
{
}

impl<R> FWoWoRegFieldBit for R
where
  R: WoWoRegFieldBit<Frt>,
  R: RegFork,
  R::Reg: FWoReg,
{
}

// }}}
// {{{ FWoWoRegFieldBits
/// Forkable multi-bit write-only field of write-only register token.
pub trait FWoWoRegFieldBits
where
  Self: WoWoRegFieldBits<Frt>,
  Self: RegFork,
  Self::Reg: FWoReg,
{
}

impl<R> FWoWoRegFieldBits for R
where
  R: WoWoRegFieldBits<Frt>,
  R: RegFork,
  R::Reg: FWoReg,
{
}

// }}}
// {{{ FRoRwRegFieldBit
/// Forkable one-bit read-only field of read-write register token.
pub trait FRoRwRegFieldBit
where
  Self: RRRegFieldBit<Frt>,
  Self: RoRRegField<Frt>,
  Self: RegFork,
  Self::Reg: FRwReg,
{
}

impl<R> FRoRwRegFieldBit for R
where
  R: RRRegFieldBit<Frt>,
  R: RoRRegField<Frt>,
  R: RegFork,
  R::Reg: FRwReg,
{
}

// }}}
// {{{ FRoRwRegFieldBits
/// Forkable multi-bit read-only field of read-write register token.
pub trait FRoRwRegFieldBits
where
  Self: RRRegFieldBits<Frt>,
  Self: RoRRegField<Frt>,
  Self: RegFork,
  Self::Reg: FRwReg,
{
}

impl<R> FRoRwRegFieldBits for R
where
  R: RRRegFieldBits<Frt>,
  R: RoRRegField<Frt>,
  R: RegFork,
  R::Reg: FRwReg,
{
}

// }}}
// {{{ FRoRoRegFieldBit
/// Forkable one-bit read-only field of read-only register token.
pub trait FRoRoRegFieldBit
where
  Self: RRRegFieldBit<Frt>,
  Self: RoRRegField<Frt>,
  Self: RegFork,
  Self::Reg: FRoReg,
{
}

impl<R> FRoRoRegFieldBit for R
where
  R: RRRegFieldBit<Frt>,
  R: RoRRegField<Frt>,
  R: RegFork,
  R::Reg: FRoReg,
{
}

// }}}
// {{{ FRoRoRegFieldBits
/// Forkable multi-bit read-only field of read-only register token.
pub trait FRoRoRegFieldBits
where
  Self: RRRegFieldBits<Frt>,
  Self: RoRRegField<Frt>,
  Self: RegFork,
  Self::Reg: FRoReg,
{
}

impl<R> FRoRoRegFieldBits for R
where
  R: RRRegFieldBits<Frt>,
  R: RoRRegField<Frt>,
  R: RegFork,
  R::Reg: FRoReg,
{
}

// }}}
// vim: set fdm=marker fmr={{{,}}} :
