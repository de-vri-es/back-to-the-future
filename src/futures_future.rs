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
	task_waker_clone,
	task_waker_wake,
	task_waker_wake_by_ref,
	task_waker_drop,
);

/// Clone the task waker from a raw pointer.
unsafe fn task_waker_clone(data: *const ()) -> RawWaker {
	let data = &*(data as *const TaskWaker);
	TaskWaker::into_raw_waker(data.task.clone())
}

/// Drop the task waker from a raw pointer.
unsafe fn task_waker_drop(data: *const ()) {
	Box::from_raw(data as *mut TaskWaker);
}

/// Wake the task waker from a raw pointer, without dropping the waker.
unsafe fn task_waker_wake_by_ref(data: *const ()) {
	let data = &*(data as *const TaskWaker);
	data.task.notify();
}

/// Wake the task waker from a raw pointer, and then drop it.
unsafe fn task_waker_wake(data: *const ()) {
	task_waker_wake_by_ref(data);
	task_waker_drop(data);
}

impl TaskWaker {
	unsafe fn into_raw_waker(task: futures::task::Task) -> RawWaker {
		let data = Box::leak(Box::new(task)) as *const _ as *const ();
		RawWaker::new(data, &TASK_WAKER_VTABLE)
	}

	fn into_waker(task: futures::task::Task) -> std::task::Waker {
		unsafe { std::task::Waker::from_raw(TaskWaker::into_raw_waker(task)) }
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
