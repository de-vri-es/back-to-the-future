// From htttps://github.com/rust-lang/rustc
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::sync::Arc;
use std::task::{
    Waker, RawWaker, RawWakerVTable,
};

macro_rules! waker_vtable {
    ($ty:ident) => {
        &RawWakerVTable {
            clone: clone_arc_raw::<$ty>,
            drop: drop_arc_raw::<$ty>,
            wake: wake_arc_raw::<$ty>,
        }
    };
}

pub trait ArcWake {
    fn wake(arc_self: &Arc<Self>);

    fn into_waker(wake: Arc<Self>) -> Waker where Self: Sized
    {
        let ptr = Arc::into_raw(wake) as *const();

        unsafe {
            Waker::new_unchecked(RawWaker::new(ptr, waker_vtable!(Self)))
        }
    }
}

unsafe fn increase_refcount<T: ArcWake>(data: *const()) {
    // Retain Arc by creating a copy
    let arc: Arc<T> = Arc::from_raw(data as *const T);
    let arc_clone = arc.clone();
    // Forget the Arcs again, so that the refcount isn't decrased
    let _ = Arc::into_raw(arc);
    let _ = Arc::into_raw(arc_clone);
}

unsafe fn clone_arc_raw<T: ArcWake>(data: *const()) -> RawWaker {
    increase_refcount::<T>(data);
    RawWaker::new(data, waker_vtable!(T))
}

unsafe fn drop_arc_raw<T: ArcWake>(data: *const()) {
    // Drop Arc
    let _: Arc<T> = Arc::from_raw(data as *const T);
}

unsafe fn wake_arc_raw<T: ArcWake>(data: *const()) {
    let arc: Arc<T> = Arc::from_raw(data as *const T);
    ArcWake::wake(&arc);
    let _ = Arc::into_raw(arc);
}
