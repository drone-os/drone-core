use super::*;

/// Wrapper for a register value that holds register reference.
pub trait RegHold<'a, T, U>
where
    Self: Sized + 'a,
    T: RegTag,
    U: Reg<T>,
{
    /// Creates a new `Hold`.
    unsafe fn new(reg: &'a U, val: U::Val) -> Self;

    /// Returns the inner value.
    fn val(&self) -> U::Val;

    /// Replaces the inner value.
    fn set_val(&mut self, val: U::Val);
}
