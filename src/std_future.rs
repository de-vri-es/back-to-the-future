//! Provides the [`std::future::Future`] interface for compatible types.
//!
//! Currently, only types implementing `futures::Future` are supported.

use std::sync::Arc;
use std::pin::Pin;

/// Adapter providing the [`std::future::Future`] interface for other future types.
pub struct FutureAdapter<Inner>(pub Inner);

#[derive(Clone)]
struct WakerNotifier {
	waker: std::task::Waker,
}

impl futures::executor::Notify for WakerNotifier {
	fn notify(&self, _id: usize) {
		self.waker.wake();
	}
}

/// New style future implementation for old style futures.
impl<F, Item, Error> FutureAdapter<futures::executor::Spawn<F>> where
	F: futures::Future<Item=Item, Error=Error>,
{
	fn poll(&mut self, waker: &std::task::Waker) -> std::task::Poll<Result<Item, Error>> {
		let notify = futures::executor::NotifyHandle::from(Arc::new(WakerNotifier{waker: waker.clone()}));

		match self.0.poll_future_notify(&notify, 0) {
			Ok(futures::Async::NotReady)   => std::task::Poll::Pending,
			Ok(futures::Async::Ready(val)) => std::task::Poll::Ready(Ok(val)),
			Err(err)                       => std::task::Poll::Ready(Err(err)),
		}
	}
}

/// New style future implementation for old style futures.
impl<F, Item, Error> std::future::Future for FutureAdapter<futures::executor::Spawn<F>> where
	F: futures::Future<Item=Item, Error=Error> + std::marker::Unpin,
{
	type Output = Result<Item, Error>;

	fn poll(self: Pin<&mut Self>, waker: &std::task::Waker) -> std::task::Poll<Self::Output> {
		Pin::get_mut(self).poll(waker)
	}
}
