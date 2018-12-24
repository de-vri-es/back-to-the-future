//! Provides the [`futures::Future`] interface for compatible types.
//!
//! Currently, only types implementing `std::future::Future` are supported.

use std::sync::Arc;
use std::pin::Pin;

/// Adapter providing the [`futures::Future`] interface for other future types.
pub struct FutureAdapter<Inner>(pub Inner);

impl<Inner> FutureAdapter<Pin<Box<Inner>>> {
	pub fn pinned(inner: Inner) -> Self {
		Self(Box::pinned(inner))
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
impl<Pointer, Future, Item, Error> futures::Future for FutureAdapter<Pin<Pointer>> where
	Pointer: std::ops::Deref<Target = Future> + std::ops::DerefMut,
	Future: std::future::Future<Output = Result<Item, Error>>,
{
	type Item  = Item;
	type Error = Error;

	fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
		let wake = unsafe { std::task::local_waker(Arc::new(TaskWaker{task: futures::task::current()})) };

		match self.0.as_mut().poll(&wake) {
			std::task::Poll::Pending         => Ok(futures::Async::NotReady),
			std::task::Poll::Ready(Ok(val))  => Ok(futures::Async::Ready(val)),
			std::task::Poll::Ready(Err(err)) => Err(err),
		}
	}
}
