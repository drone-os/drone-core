use super::*;

/// Wrapper for a register value that holds register reference.
pub trait RegHold<'a, T, U>
where
  Self: Sized,
  T: RegTag + 'a,
  U: Reg<'a, T>,
{
  /// Type that wraps a raw register value.
  type Val: RegVal;

  #[doc(hidden)]
  unsafe fn hold(reg: &'a U, val: Self::Val) -> Self;

  /// Returns the inner value.
  fn val(&self) -> Self::Val;

  /// Replaces the inner value.
  fn set_val(&mut self, val: Self::Val);
}
