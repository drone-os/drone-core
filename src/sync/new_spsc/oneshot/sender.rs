use core::{
    marker::PhantomData,
    task::{Context, Poll},
};

/// The sending-half of [`oneshot::channel`](super::channel).
pub struct Sender<T> {
    _marker: PhantomData<T>,
}

impl<T> Sender<T> {
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
    pub fn send(self, _data: T) -> Result<(), T> {
        todo!()
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
    pub fn poll_canceled(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        todo!()
    }

    /// Tests to see whether this `Sender`'s corresponding `Receiver` has been
    /// dropped.
    ///
    /// Unlike [`poll_canceled`](Sender::poll_canceled), this function does not
    /// enqueue a task for wakeup upon cancellation, but merely reports the
    /// current state, which may be subject to concurrent modification.
    #[inline]
    pub fn is_canceled(&self) -> bool {
        todo!()
    }
}

impl<T> Drop for Sender<T> {
    #[inline]
    fn drop(&mut self) {
        todo!()
    }
}
