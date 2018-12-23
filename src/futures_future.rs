use std::sync::Arc;
use std::pin::Pin;

pub struct FutureAdapter<Inner> {
	inner: Pin<Inner>,
}

impl<Inner: std::ops::Deref> FutureAdapter<Inner> {
	pub fn new(inner: Pin<Inner>) -> Self {
		Self{inner}
	}

	pub fn inner(&self) -> Pin<&Inner::Target> {
		self.inner.as_ref()
	}
}

impl<Inner> FutureAdapter<Box<Inner>> {
	pub fn pinned(inner: Inner) -> Self {
		Self{inner: Box::pinned(inner)}
	}
}

impl<Inner: std::ops::Deref + std::ops::DerefMut> FutureAdapter<Inner> {
	pub fn inner_mut(&mut self) -> Pin<&mut Inner::Target> {
		self.inner.as_mut()
	}
}

impl<Inner> FutureAdapter<Inner> {
	pub fn into_inner(self) -> Pin<Inner> {
		self.inner
	}
}

struct TaskWaker {
	task: futures::task::Task,
}

impl std::task::Wake for TaskWaker {
	fn wake(arc_self: &Arc<Self>) {
		arc_self.task.notify();
	}
}

/// Old style future implementation for new style futures.
impl<X, F, Item, Error> futures::Future for FutureAdapter<X> where
	X: std::ops::Deref<Target = F> + std::ops::DerefMut,
	F: std::future::Future<Output = Result<Item, Error>>,
{
	type Item  = Item;
	type Error = Error;

	fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
		let wake = unsafe { std::task::local_waker(Arc::new(TaskWaker{task: futures::task::current()})) };

		match self.inner_mut().poll(&wake) {
			std::task::Poll::Pending         => Ok(futures::Async::NotReady),
			std::task::Poll::Ready(Ok(val))  => Ok(futures::Async::Ready(val)),
			std::task::Poll::Ready(Err(err)) => Err(err),
		}
	}
}
