[![Documentation](https://docs.rs/back-to-the-future/badge.svg)](https://docs.rs/back-to-the-future)
[![crates.io](https://img.shields.io/crates/v/back-to-the-future.svg)](https://crates.io/crates/back-to-the-future)
[![Build Status](https://travis-ci.org/de-vri-es/back-to-the-future.svg?branch=master)](https://travis-ci.org/de-vri-es/back-to-the-future)

# `std` and `futures` interoperability
This crate implements adapters for the two different future types: `std::future::Future` and `futures::Future`.
You can should be able to seamlessly convert the one into the other.
The aim if to be able to use new async/await syntax with existing `futures::Future` infrastructure.

Keep in mind that much of the used features are still unstable and only available on nightly with feature gates.

A simple example:
```
#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use std::time::{Duration, Instant};
use tokio::timer::Delay;

use back_to_the_future::futures_await;
use back_to_the_future::BoxIntoFutures;
use back_to_the_future::IntoFutures;

fn main() {
	let f = async {
		futures_await!(Delay::new(Instant::now() + Duration::new(0, 10))).unwrap();
		Ok(())
	};
	tokio::run(f.box_into_futures());
}
```
