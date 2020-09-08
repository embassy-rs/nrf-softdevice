use core::cell::UnsafeCell;
use core::future::Future;

use core::mem::MaybeUninit;
use core::pin::Pin;
use core::ptr;
use core::task::{Context, Poll};

use crate::util::*;

pub struct Portal<T> {
    inner: UnsafeCell<Inner<T>>,
}

struct Inner<T> {
    // This can be optimized a bit by directly using a Waker, since we know
    // the waker is present iff value is not null
    waker: WakerStore,
    value: *mut T,
}

unsafe impl<T> Send for Portal<T> {}
unsafe impl<T> Sync for Portal<T> {}

fn assert_thread_mode() {
    deassert!(
        cortex_m::peripheral::SCB::vect_active()
            == cortex_m::peripheral::scb::VectActive::ThreadMode,
        "portals are not usable from interrupts"
    );
}

impl<T> Portal<T> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(Inner {
                waker: WakerStore::new(),
                value: ptr::null_mut(),
            }),
        }
    }

    pub fn signal(&self, val: T) {
        assert_thread_mode();

        // safety: this runs from thread mode
        let this = unsafe { &mut *self.inner.get() };

        if !this.value.is_null() {
            unsafe { this.value.write(val) };
            this.value = ptr::null_mut();
            this.waker.wake()
        }
    }

    pub fn signal_with(&self, f: impl FnOnce() -> T) {
        assert_thread_mode();

        // safety: this runs from thread mode
        let this = unsafe { &mut *self.inner.get() };

        if !this.value.is_null() {
            unsafe { this.value.write(f()) };
            this.value = ptr::null_mut();
            this.waker.wake()
        }
    }

    pub fn wait<'a>(&'a self) -> impl Future<Output = T> + 'a {
        assert_thread_mode();
        WaitFuture {
            linked: false,
            data: MaybeUninit::uninit(),
            portal: self,
        }
    }
}

struct WaitFuture<'a, T> {
    linked: bool,
    data: MaybeUninit<T>,
    portal: &'a Portal<T>,
}

impl<'a, T> Drop for WaitFuture<'a, T> {
    fn drop(&mut self) {
        if self.linked {
            // safety: this must be from thread mode since Portal.wait runs in
            // thread mode and the future is not Send.
            let portal = unsafe { &mut *self.portal.inner.get() };

            portal.value = ptr::null_mut()
        }
    }
}

impl<'a, T> Future for WaitFuture<'a, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        // safety: this must be from thread mode since Portal.wait runs in
        // thread mode and the future is not Send.
        let portal = unsafe { &mut *self.portal.inner.get() };

        let this = unsafe { self.get_unchecked_mut() };

        if !this.linked {
            if !portal.value.is_null() {
                depanic!("portal is already in use (another task is waiting)")
            }
            portal.value = this.data.as_mut_ptr();
            portal.waker.store(cx.waker());
            this.linked = true;
        }

        if portal.value.is_null() {
            Poll::Ready(unsafe { this.data.as_mut_ptr().read() })
        } else {
            portal.waker.store(cx.waker());
            Poll::Pending
        }
    }
}
