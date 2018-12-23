[![Documentation](https://docs.rs/back-to-the-future/badge.svg)](https://docs.rs/back-to-the-future)
[![crates.io](https://img.shields.io/crates/v/back-to-the-future.svg)](https://crates.io/crates/back-to-the-future)
[![Build Status](https://travis-ci.org/de-vri-es/back-to-the-future.svg?branch=master)](https://travis-ci.org/de-vri-es/back-to-the-future)

# back-to-the-future

## `std` and `futures` interoperability
This crate implements adapters for the two different future types: [`std::future::Future`] and [`futures::Future`].
You can seamlessly convert the one into the other.
The aim is to be able to use new async/await syntax with existing [`futures::Future`] infrastructure, such as tokio.

Keep in mind that many of the used features are still unstable and only available on nightly with feature gates.

A simple example:
```rust
#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use std::time::{Duration, Instant};
use tokio::timer::Delay;

use back_to_the_future::{futures_await, BoxIntoFutures};

fn main() {
  let f = async {
    // Await an old futures::Future using the futures_await! macro.
    // This macro wraps the future in an adapter behind the scenes.
    futures_await!(Delay::new(Instant::now() + Duration::new(0, 10))).unwrap();
    Ok(())
  };

  // Convert the std::future::Future into a futures::Future so that tokio::run can use it.
  tokio::run(f.box_into_futures());
}
```
