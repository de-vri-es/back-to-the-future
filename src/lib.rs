//! # `std` and `futures` interoperability
//! This crate implements adapters for the two different future types: [`std::future::Future`] and [`futures::Future`].
//! You can seamlessly convert the one into the other.
//! The aim is to be able to use new async/await syntax with existing [`futures::Future`] infrastructure, such as tokio.
//!
//! Keep in mind that many of the used features are still unstable and only available on nightly with feature gates.
//!
//! A simple example:
//! ```
//! #![feature(async_await)]
//!
//! use std::time::{Duration, Instant};
//! use tokio::timer::Delay;
//!
//! use back_to_the_future::{futures_await, BoxIntoFutures};
//!
//! fn main() {
//!   let f = async {
//!     // Await an old futures::Future using the futures_await! macro.
//!     // This macro wraps the future in an adapter behind the scenes.
//!     futures_await!(Delay::new(Instant::now() + Duration::new(0, 10))).unwrap();
//!     Ok(())
//!   };
//!
//!   // Convert the std::future::Future into a futures::Future so that tokio::run can use it.
//!   tokio::run(f.box_into_futures());
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/back_to_the_future/0.1.3")]

#![feature(async_await)]

pub mod std_future;
pub mod futures_future;

use std::pin::Pin;

#[macro_export]
/// Await a futures::Future by first wrapping it in an std::future::Future adapter.
macro_rules! futures_await {
	($ex:expr) => { $crate::IntoStdFuture::into_std_future($ex).await };
}

/// Conversion of non-std futures into `std::future::Future`.
pub trait IntoStdFuture {
	type Output;

	fn into_std_future(self) -> std_future::FutureAdapter<Self::Output>;
}

/// Conversion of non-futures future into `futures::Future`.
pub trait IntoFutures {
	type Output;

	fn into_futures(self) -> futures_future::FutureAdapter<Self::Output>;
}

/// Conversion of any non-futures future into `futures::Future`.
///
/// This is very similar to the `IntoFutures` trait, except that it will first
/// box the future to circumvent any movability and lifetime requirements.
pub trait BoxIntoFutures {
	type Output;

	fn box_into_futures(self) -> futures_future::FutureAdapter<Self::Output>;
}

impl<T: futures::IntoFuture> IntoStdFuture for T {
	type Output = futures::executor::Spawn<T::Future>;

	/// Convert any [`futures::Future`] into a [`std::future::Future`].
	fn into_std_future(self) -> std_future::FutureAdapter<Self::Output> {
		std_future::FutureAdapter(futures::executor::spawn(self.into_future()))
	}
}

impl<P, F> IntoFutures for Pin<P> where
	P: std::ops::Deref<Target = F>,
	F: std::future::Future,
{
	type Output = Pin<P>;

	/// Convert a pinned [`std::future::Future`] into a [`futures::Future`].
	fn into_futures(self) -> futures_future::FutureAdapter<Self::Output> {
		futures_future::FutureAdapter(self)
	}
}

impl<F> BoxIntoFutures for F where
	F: std::future::Future,
{
	type Output = Pin<Box<F>>;

	/// Convert any [`std::future::Future`] into a [`futures::Future`].
	///
	/// To enable the conversion, it is boxed and pinned.
	/// If your future is already pinned, prefer using [`IntoFutures`]
	fn box_into_futures(self) -> futures_future::FutureAdapter<Self::Output> {
		futures_future::FutureAdapter(Box::pin(self))
	}
}
