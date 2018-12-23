//! # `std` and `futures` interoperability
//! This crate implements adapters for the two different future types: [`std::future::Future`] and [`futures::Future`].
//! You can should be able to seamlessly convert the one into the other.
//! The aim if to be able to use new async/await syntax with existing [`futures::Future`] infrastructure.
//!
//! Keep in mind that much of the used features are still unstable and only available on nightly with feature gates.
//!
//! A simple example:
//! ```
//! #![feature(async_await)]
//! #![feature(await_macro)]
//! #![feature(futures_api)]
//!
//! use std::time::{Duration, Instant};
//! use tokio::timer::Delay;
//!
//! use back_to_the_future::futures_await;
//! use back_to_the_future::BoxIntoFutures;
//! use back_to_the_future::IntoFutures;
//!
//! fn main() {
//! 	let f = async {
//! 		futures_await!(Delay::new(Instant::now() + Duration::new(0, 10))).unwrap();
//! 		Ok(())
//! 	};
//! 	tokio::run(f.box_into_futures());
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/back_to_the_future/0.1.0")]

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

use std::pin::Pin;

#[macro_export]
/// Macro to await a futures::Future by first wrapping it in an std::future::Future adapter.
macro_rules! futures_await {
	($ex:expr) => { await!($crate::std_future::FutureAdapter::new($ex)) };
}

/// Trait to convert non-std futures into `std::Future`.
pub trait IntoStdFuture {
	type Output;

	fn into_std_future(self) -> std_future::FutureAdapter<Self::Output>;
}

/// Trait to box, pin and convert a non-futures future into `futures::Future`.
///
/// This is very similar to the `IntoFutures` crate,
/// except that it will first call `Box::pinned` to pin the future.
pub trait BoxIntoFutures {
	type Output;

	fn box_into_futures(self) -> futures_future::FutureAdapter<Self::Output>;
}

/// Trait to convert a pinned non-futures future into `futures::Future`.
pub trait IntoFutures {
	type Output;

	fn into_futures(self) -> futures_future::FutureAdapter<Self::Output>;
}

/// Convert a futures::Future into `std::Future`.
impl<T: futures::IntoFuture> IntoStdFuture for T {
	type Output = T::Future;

	fn into_std_future(self) -> std_future::FutureAdapter<Self::Output> {
		std_future::FutureAdapter::new(self.into_future())
	}
}

/// Box, pin and convert an `std::future::Future` into a `futures::Future`.
impl<F> BoxIntoFutures for F where
	F: std::future::Future,
{
	type Output = Pin<Box<F>>;

	fn box_into_futures(self) -> futures_future::FutureAdapter<Self::Output> {
		futures_future::FutureAdapter::new(Box::pinned(self))
	}
}

/// Convert an `std::future::Future` into a `futures::Future`.
impl<P, F> IntoFutures for Pin<P> where
	P: std::ops::Deref<Target = F>,
	F: std::future::Future,
{
	type Output = Pin<P>;

	fn into_futures(self) -> futures_future::FutureAdapter<Self::Output> {
		futures_future::FutureAdapter::new(self)
	}
}
