use super::Inner;
use alloc::sync::Arc;
use futures::prelude::*;
use sync::spsc::SpscInner;

/// The sending-half of [`oneshot::channel`](super::channel).
pub struct Sender<T, E> {
  inner: Arc<Inner<T, E>>,
}

impl<T, E> Sender<T, E> {
  #[inline(always)]
  pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
    Self { inner }
  }

  /// Completes this oneshot with a result.
  ///
  /// If the value is successfully enqueued, then `Ok(())` is returned. If the
  /// receiving end was dropped before this function was called, then `Err` is
  /// returned with the value provided.
  #[inline(always)]
  pub fn send(self, data: Result<T, E>) -> Result<(), Result<T, E>> {
    self.inner.send(data)
  }

  /// Polls this [`Sender`] half to detect whether the [`Receiver`] this has
  /// paired with has gone away.
  ///
  /// # Panics
  ///
  /// Like `Future::poll`, this function will panic if it's not called from
  /// within the context of a task. In other words, this should only ever be
  /// called from inside another future.
  ///
  /// If you're calling this function from a context that does not have a task,
  /// then you can use the [`is_canceled`] API instead.
  ///
  /// [`Sender`]: Sender
  /// [`Receiver`]: super::Receiver
  /// [`is_canceled`]: Sender::is_canceled
  #[inline(always)]
  pub fn poll_cancel(&mut self, cx: &mut task::Context) -> Poll<(), ()> {
    self.inner.poll_cancel(cx)
  }

  /// Tests to see whether this [`Sender`]'s corresponding [`Receiver`] has gone
  /// away.
  ///
  /// [`Sender`]: Sender
  /// [`Receiver`]: super::Receiver
  #[inline(always)]
  pub fn is_canceled(&self) -> bool {
    self.inner.is_canceled()
  }
}

impl<T, E> Drop for Sender<T, E> {
  #[inline(always)]
  fn drop(&mut self) {
    self.inner.drop_tx();
  }
}

impl<T, E> Inner<T, E> {
  fn send(&self, data: Result<T, E>) -> Result<(), Result<T, E>> {
    if self.is_canceled() {
      Err(data)
    } else {
      unsafe { *self.data.get() = Some(data) };
      Ok(())
    }
  }
}
