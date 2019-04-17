//! Provides the [`futures::Future`] interface for compatible types.
//!
//! Currently, only types implementing `std::future::Future` are supported.
use std::pin::Pin;

use std::task::{RawWaker, RawWakerVTable};

/// Adapter providing the [`futures::Future`] interface for other future types.
pub struct FutureAdapter<Inner>(pub Inner);

impl<Inner> FutureAdapter<Pin<Box<Inner>>> {
	pub fn pin(inner: Inner) -> Self {
		Self(Box::pin(inner))
	}
}

struct TaskWaker {
	task: futures::task::Task,
}

const TASK_WAKER_VTABLE : RawWakerVTable = RawWakerVTable::new(
	clone_task_waker,
	drop_task_waker,
	wake_task_waker,
);

unsafe fn clone_task_waker(data: *const ()) -> RawWaker {
	let data = &*(data as *const TaskWaker);
	TaskWaker::into_raw_waker(data.task.clone())
}

unsafe fn drop_task_waker(data: *const ()) {
	Box::from_raw(data as *mut TaskWaker);
}

unsafe fn wake_task_waker(data: *const ()) {
	let data = &*(data as *const TaskWaker);
	data.task.notify();
}

impl TaskWaker {
	unsafe fn into_raw_waker(task: futures::task::Task) -> RawWaker {
		let data = Box::leak(Box::new(task)) as *const _ as *const ();
		RawWaker::new(data, &TASK_WAKER_VTABLE)
	}

	fn into_waker(task: futures::task::Task) -> std::task::Waker {
		unsafe { std::task::Waker::new_unchecked(TaskWaker::into_raw_waker(task)) }
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
		let waker = TaskWaker::into_waker(futures::task::current());
		let mut context = std::task::Context::from_waker(&waker);

		match self.0.as_mut().poll(&mut context) {
			std::task::Poll::Pending         => Ok(futures::Async::NotReady),
			std::task::Poll::Ready(Ok(val))  => Ok(futures::Async::Ready(val)),
			std::task::Poll::Ready(Err(err)) => Err(err),
		}
	}
}
