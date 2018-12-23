use std::sync::Arc;
use std::pin::Pin;

pub struct FutureAdapter<Inner> {
	inner: Inner,
}

impl<Inner> FutureAdapter<Inner> {
	pub fn new(inner: Inner) -> Self {
		Self{inner}
	}

	pub fn inner(&self) -> &Inner {
		&self.inner
	}

	pub fn inner_mut(&mut self) -> &mut Inner {
		&mut self.inner
	}

	pub fn into_inner(self) -> Inner {
		self.inner
	}
}

impl<Inner> FutureAdapter<Pin<Box<Inner>>> {
	pub fn pinned(inner: Inner) -> Self {
		Self{inner: Box::pinned(inner)}
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

		match self.inner_mut().as_mut().poll(&wake) {
			std::task::Poll::Pending         => Ok(futures::Async::NotReady),
			std::task::Poll::Ready(Ok(val))  => Ok(futures::Async::Ready(val)),
			std::task::Poll::Ready(Err(err)) => Err(err),
		}
	}
}
