use core::cell::UnsafeCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::waker_store::WakerStore;

pub struct Signal<T> {
    inner: UnsafeCell<Inner<T>>,
}

struct Inner<T> {
    waker: WakerStore,
    value: Option<T>,
}

unsafe impl<T: Send> Send for Signal<T> {}
unsafe impl<T: Send> Sync for Signal<T> {}

impl<T: Send> Signal<T> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(Inner {
                waker: WakerStore::new(),
                value: None,
            }),
        }
    }

    pub fn signal(&self, val: T) {
        unsafe {
            crate::interrupt::raw_free(|| {
                let this = &mut *self.inner.get();
                this.value = Some(val);
                this.waker.wake();
            })
        }
    }

    pub fn wait<'a>(&'a self) -> impl Future<Output = T> + 'a {
        WaitFuture { signal: self }
    }
}

struct WaitFuture<'a, T> {
    signal: &'a Signal<T>,
}

impl<'a, T: Send> Future for WaitFuture<'a, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        unsafe {
            crate::interrupt::raw_free(|| {
                let this = &mut *self.signal.inner.get();
                if let Some(val) = this.value.take() {
                    Poll::Ready(val)
                } else {
                    this.waker.store(cx.waker());
                    Poll::Pending
                }
            })
        }
    }
}
