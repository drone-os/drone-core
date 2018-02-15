use futures;
use io;

/// I/O operation future.
pub trait Future {
  /// I/O session.
  type Sess;

  /// I/O result reader.
  type Resp = io::NoResp;

  /// I/O error.
  type Error;

  /// I/O equivalent of `Future::poll`.
  fn poll(&mut self) -> io::Poll<Self>;
}

/// A type returned from [`io::Future::poll`](Future::poll).
pub type Poll<F> = futures::Poll<
  (<F as io::Future>::Sess, <F as io::Future>::Resp),
  (<F as io::Future>::Sess, <F as io::Future>::Error),
>;

impl<'s, F, S, R, E> io::Future for F
where
  F: futures::Future<Item = (S, R), Error = (S, E)>,
  R: io::Responder<'s, S>,
  S: 's,
{
  type Sess = S;
  type Resp = R;
  type Error = E;

  #[inline(always)]
  fn poll(&mut self) -> io::Poll<Self> {
    futures::Future::poll(self)
  }
}
