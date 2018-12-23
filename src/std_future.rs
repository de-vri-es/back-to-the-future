use std::sync::Arc;
use std::pin::Pin;

pub struct FutureAdapter<Inner> {
	inner: futures::executor::Spawn<Inner>,
}

impl<Inner> FutureAdapter<Inner> {
	pub fn new(inner: Inner) -> Self {
		Self{inner: futures::executor::spawn(inner)}
	}

	pub fn inner(&self) -> &Inner {
		self.inner.get_ref()
	}

	pub fn inner_mut(&mut self) -> &mut Inner {
		self.inner.get_mut()
	}

	pub fn into_inner(self) -> Inner {
		self.inner.into_inner()
	}
}

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
impl<F, Item, Error> FutureAdapter<F> where
	F: futures::Future<Item=Item, Error=Error>,
{
	fn poll(&mut self, waker: &std::task::LocalWaker) -> std::task::Poll<Result<Item, Error>> {
		let notify = futures::executor::NotifyHandle::from(Arc::new(WakerNotifier{waker: waker.clone().into_waker()}));

		match self.inner.poll_future_notify(&notify, 0) {
			Ok(futures::Async::NotReady)   => std::task::Poll::Pending,
			Ok(futures::Async::Ready(val)) => std::task::Poll::Ready(Ok(val)),
			Err(err)                       => std::task::Poll::Ready(Err(err)),
		}
	}
}

/// New style future implementation for old style futures.
impl<F, Item, Error> std::future::Future for FutureAdapter<F> where
	F: futures::Future<Item=Item, Error=Error> + std::pin::Unpin,
{
	type Output = Result<Item, Error>;

	fn poll(self: Pin<&mut Self>, waker: &std::task::LocalWaker) -> std::task::Poll<Self::Output> {
		Pin::get_mut(self).poll(waker)
	}
}
