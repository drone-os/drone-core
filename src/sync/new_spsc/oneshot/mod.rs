//! A channel for sending a single message between asynchronous tasks.
//!
//! See [`channel`] constructor for more.

mod receiver;
mod sender;

pub use self::{
    receiver::{Canceled, Receiver},
    sender::Sender,
};

/// Creates a new one-shot channel, returning the sender/receiver halves.
///
/// The [`Sender`] half is used to signal the end of a computation and provide
/// its value. The [`Receiver`] half is a [`Future`](core::future::Future)
/// resolving to the value that was given to the [`Sender`] half.
#[inline]
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    todo!()
}
