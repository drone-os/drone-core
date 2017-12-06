/// Returns a result of the future. Should be used inside [`AsyncFuture`]
/// context.
///
/// [`AsyncFuture`]: struct.AsyncFuture.html
pub macro await($future:expr) {
  {
    let mut future = $future;
    loop {
      match future.poll() {
        Ok(Async::NotReady) => (),
        Ok(Async::Ready(ready)) => break Ok(ready),
        Err(err) => break Err(err),
      }
      yield;
    }
  }
}
