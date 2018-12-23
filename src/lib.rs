#![feature(arbitrary_self_types)]
#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]
#![feature(never_type)]
#![feature(pin)]
#![feature(trait_alias)]
#![feature(specialization)]

pub mod std_future;
pub mod futures_future;

#[macro_export]
macro_rules! futures_await {
	($ex:expr) => { await!($crate::std_future::FutureAdapter::new($ex)) };
}

/// Trait to convert non-std futures into `std::Future`.
pub trait IntoStdFuture {
	type Output;

	fn into_std_future(self) -> std_future::FutureAdapter<Self::Output>;
}

impl<T: futures::IntoFuture> IntoStdFuture for T {
	type Output = T::Future;

	fn into_std_future(self) -> std_future::FutureAdapter<Self::Output> {
		std_future::FutureAdapter::new(self.into_future())
	}
}

/// Trait to convert non-futures futures into `futures::Future`.
pub trait BoxIntoFutures {
	type Output;

	fn box_into_futures(self) -> futures_future::FutureAdapter<Self::Output>;
}

/// Trait to convert a pinned non-futures futures into `futures::Future`.
pub trait IntoFutures {
	type Output;

	fn into_futures(self) -> futures_future::FutureAdapter<Self::Output>;
}

impl<F> BoxIntoFutures for F where
	F: std::future::Future,
{
	type Output = Box<F>;

	fn box_into_futures(self) -> futures_future::FutureAdapter<Self::Output> {
		futures_future::FutureAdapter::new(Box::pinned(self))
	}
}

impl<P, F> IntoFutures for std::pin::Pin<P> where
	P: std::ops::Deref<Target = F>,
	F: std::future::Future,
{
	type Output = P;

	fn into_futures(self) -> futures_future::FutureAdapter<Self::Output> {
		futures_future::FutureAdapter::new(self)
	}
}
