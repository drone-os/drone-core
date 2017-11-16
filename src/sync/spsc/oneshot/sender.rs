use super::Inner;
use alloc::arc::Arc;
use futures::Poll;
use sync::spsc::SpscInner;

/// The sending-half of [`oneshot::channel`].
///
/// [`oneshot::channel`]: fn.channel.html
pub struct Sender<R, E> {
  inner: Arc<Inner<R, E>>,
}

impl<R, E> Sender<R, E> {
  #[inline(always)]
  pub(super) fn new(inner: Arc<Inner<R, E>>) -> Self {
    Self { inner }
  }

  /// Completes this oneshot with a result.
  ///
  /// If the value is successfully enqueued, then `Ok(())` is returned. If the
  /// receiving end was dropped before this function was called, then `Err` is
  /// returned with the value provided.
  ///
  /// [`Receiver`]: struct.Receiver.html
  #[inline]
  pub fn send(self, data: Result<R, E>) -> Result<(), Result<R, E>> {
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
  /// [`Sender`]: struct.Sender.html
  /// [`Receiver`]: struct.Receiver.html
  /// [`is_canceled`]: struct.Receiver.html#method.is_canceled
  #[inline]
  pub fn poll_cancel(&mut self) -> Poll<(), ()> {
    self.inner.poll_cancel()
  }

  /// Tests to see whether this [`Sender`]'s corresponding [`Receiver`] has gone
  /// away.
  ///
  /// [`Sender`]: struct.Sender.html
  /// [`Receiver`]: struct.Receiver.html
  #[inline(always)]
  pub fn is_canceled(&self) -> bool {
    self.inner.is_canceled()
  }
}

impl<R, E> Drop for Sender<R, E> {
  #[inline]
  fn drop(&mut self) {
    self.inner.drop_tx();
  }
}

impl<R, E> Inner<R, E> {
  #[inline(always)]
  fn send(&self, data: Result<R, E>) -> Result<(), Result<R, E>> {
    if self.is_canceled() {
      Err(data)
    } else {
      unsafe { *self.data.get() = Some(data) };
      Ok(())
    }
  }
}
