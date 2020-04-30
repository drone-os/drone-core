use super::Inner;
use crate::sync::spsc::SpscInner;
use alloc::sync::Arc;
use core::{
    sync::atomic::Ordering,
    task::{Context, Poll},
};

const IS_TX_HALF: bool = true;

/// The sending-half of [`oneshot::channel`](super::channel).
pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub(super) fn new(inner: Arc<Inner<T>>) -> Self {
        Self { inner }
    }

    /// Completes this oneshot with a successful result.
    ///
    /// This function will consume `self` and indicate to the other end, the
    /// [`Receiver`](super::Receiver), that the value provided is the result of
    /// the computation this represents.
    ///
    /// If the value is successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end was dropped before this
    /// function was called, however, then `Err` is returned with the value
    /// provided.
    #[inline]
    pub fn send(self, data: T) -> Result<(), T> {
        self.inner.send(data)
    }

    /// Polls this `Sender` half to detect whether its associated
    /// [`Receiver`](super::Receiver) with has been dropped.
    ///
    /// # Return values
    ///
    /// If `Ok(Ready)` is returned then the associated `Receiver` has been
    /// dropped, which means any work required for sending should be canceled.
    ///
    /// If `Ok(Pending)` is returned then the associated `Receiver` is still
    /// alive and may be able to receive a message if sent. The current task,
    /// however, is scheduled to receive a notification if the corresponding
    /// `Receiver` goes away.
    #[inline]
    pub fn poll_canceled(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.inner.poll_half(
            cx,
            IS_TX_HALF,
            Ordering::Relaxed,
            Ordering::Release,
            Inner::take_cancel,
        )
    }

    /// Tests to see whether this `Sender`'s corresponding `Receiver` has been
    /// dropped.
    ///
    /// Unlike [`poll_canceled`](Sender::poll_canceled), this function does not
    /// enqueue a task for wakeup upon cancellation, but merely reports the
    /// current state, which may be subject to concurrent modification.
    #[inline]
    pub fn is_canceled(&self) -> bool {
        self.inner.is_canceled(Ordering::Relaxed)
    }
}

impl<T> Drop for Sender<T> {
    #[inline]
    fn drop(&mut self) {
        self.inner.close_half(IS_TX_HALF);
    }
}

impl<T> Inner<T> {
    fn send(&self, data: T) -> Result<(), T> {
        if self.is_canceled(Ordering::Relaxed) {
            Err(data)
        } else {
            unsafe { *self.data.get() = Some(data) };
            Ok(())
        }
    }
}
