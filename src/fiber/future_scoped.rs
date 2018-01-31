use core::mem;
use sync::spsc::oneshot::{channel, Receiver};
use thread::prelude::*;

/// A future for result from another thread.
#[must_use]
pub struct FiberFutureScoped<'scope, R, E, S: 'scope> {
  rx: Receiver<ThreadScopeGuard<'scope, S, Result<R, E>>, !>,
}

impl<'scope, R, E, S: 'scope> FiberFutureScoped<'scope, R, E, S> {
  pub(crate) fn new<T, U, G>(
    scope: ThreadScope<'scope, T, U, S>,
    mut generator: G,
  ) -> Self
  where
    T: ThreadToken<U>,
    U: ThreadTag,
    G: Generator<Yield = (), Return = Result<R, E>>,
    G: Send + 'scope,
    R: Send + 'scope,
    E: Send + 'scope,
  {
    let (tx, rx) = channel();
    let (thread, token) = scope.into_parts();
    unsafe {
      thread.fibers().add_scoped(move || loop {
        match generator.resume() {
          Yielded(()) => {}
          Complete(value) => match tx.send(Ok(token.wrap(value))) {
            Ok(()) => break,
            Err(_value) => unreachable!(),
          },
        }
        yield;
      });
    }
    Self { rx }
  }

  /// Gracefully close this future, preventing sending any future messages.
  #[inline(always)]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<'scope, R, E, S: 'scope> Future for FiberFutureScoped<'scope, R, E, S> {
  type Item = ThreadScopeGuard<'scope, S, Result<R, E>>;
  type Error = !;

  #[inline(always)]
  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    match self.rx.poll() {
      Ok(value) => Ok(value),
      Err(_) => unsafe { mem::unreachable() },
    }
  }
}
