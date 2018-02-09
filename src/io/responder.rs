/// An object, which returns a result of the last I/O operation.
pub trait Responder<'s, S: 's> {
  /// Final result type of the I/O operation.
  type Output = ();

  /// Returns a result from `sess`.
  fn respond(self, sess: &'s S) -> Self::Output;
}

/// A responder, which do nothing and returns `()`.
pub struct NoResp;

impl<'s, T, S, O> Responder<'s, S> for T
where
  T: FnOnce(&'s S) -> O,
  S: 's,
{
  type Output = O;

  #[inline(always)]
  fn respond(self, sess: &'s S) -> O {
    self(sess)
  }
}

impl<'s, S: 's> Responder<'s, S> for NoResp {
  type Output = ();

  #[inline(always)]
  fn respond(self, _sess: &'s S) {}
}
