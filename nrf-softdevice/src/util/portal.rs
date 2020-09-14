use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::mem::MaybeUninit;

use crate::util::*;

/// Utility to call a closure across tasks.
pub struct Portal<T> {
    inner: UnsafeCell<Inner<T>>,
}

struct Inner<T> {
    waker: WakerStore,

    // Option because you can't have null wide pointers.
    func: Option<*mut dyn FnMut(T)>,
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
                func: None,
            }),
        }
    }

    pub fn call(&self, val: T) {
        assert_thread_mode();

        // safety: this runs from thread mode
        let this = unsafe { &mut *self.inner.get() };

        if let Some(func) = this.func {
            let func = unsafe { &mut *func };
            this.func = None; // Remove func before calling it, to avoid reentrant calls.
            func(val);
            this.waker.wake()
        }
    }

    pub fn wait<'a, R, F>(&'a self, mut func: F) -> impl Future<Output = R> + 'a
    where
        F: FnMut(T) -> R + 'a,
    {
        assert_thread_mode();

        async move {
            let bomb = DropBomb::new();

            let signal = Signal::new();
            let mut result: MaybeUninit<R> = MaybeUninit::uninit();
            let mut call_func = |val: T| {
                unsafe { result.as_mut_ptr().write(func(val)) };
                signal.signal(());
            };

            // safety: this runs from thread mode
            let this = unsafe { &mut *self.inner.get() };

            let func_ptr: *mut dyn FnMut(T) = &mut call_func as _;
            let func_ptr: *mut dyn FnMut(T) = unsafe { mem::transmute(func_ptr) };
            this.func = Some(func_ptr);

            signal.wait().await;

            bomb.defuse();

            unsafe { result.assume_init() }
        }
    }
}
