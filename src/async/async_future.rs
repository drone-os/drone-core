/// A generator-based future.
#[must_use]
pub struct AsyncFuture<G, R, E>(G)
where
  G: Generator<Yield = (), Return = Result<R, E>>;

impl<G, R, E> AsyncFuture<G, R, E>
where
  G: Generator<Yield = (), Return = Result<R, E>>,
{
  /// Creates a new `AsyncFuture`.
  #[inline(always)]
  pub fn new(gen: G) -> Self {
    AsyncFuture(gen)
  }
}

impl<G, R, E> Future for AsyncFuture<G, R, E>
where
  G: Generator<Yield = (), Return = Result<R, E>>,
{
  type Item = R;
  type Error = E;

  #[inline(always)]
  fn poll(&mut self) -> Poll<R, E> {
    match self.0.resume() {
      Yielded(()) => Ok(Async::NotReady),
      Complete(complete) => complete.map(Async::Ready),
    }
  }
}
