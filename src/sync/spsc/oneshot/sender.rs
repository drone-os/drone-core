use super::Inner;
use alloc::arc::Arc;
use futures::Poll;
use sync::spsc::SpscInner;

/// The sending-half of [`oneshot::channel`].
///
/// [`oneshot::channel`]: fn.channel.html
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
  ///
  /// [`Receiver`]: struct.Receiver.html
  #[inline]
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

impl<T, E> Drop for Sender<T, E> {
  #[inline]
  fn drop(&mut self) {
    self.inner.drop_tx();
  }
}

impl<T, E> Inner<T, E> {
  #[inline(always)]
  fn send(&self, data: Result<T, E>) -> Result<(), Result<T, E>> {
    if self.is_canceled() {
      Err(data)
    } else {
      unsafe { *self.data.get() = Some(data) };
      Ok(())
    }
  }
}
