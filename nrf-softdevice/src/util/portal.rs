use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::mem::MaybeUninit;

use crate::util::{OnDrop, Signal};

/// Utility to call a closure across tasks.
pub struct Portal<T> {
    state: UnsafeCell<State<T>>,
}

enum State<T> {
    None,
    Running,
    Waiting(*mut dyn FnMut(T)),
    Done,
}

unsafe impl<T> Send for Portal<T> {}
unsafe impl<T> Sync for Portal<T> {}

fn assert_thread_mode() {
    assert!(
        cortex_m::peripheral::SCB::vect_active()
            == cortex_m::peripheral::scb::VectActive::ThreadMode,
        "portals are not usable from interrupts"
    );
}

impl<T> Portal<T> {
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(State::None),
        }
    }

    pub fn call(&self, val: T) -> bool {
        assert_thread_mode();

        // safety: this runs from thread mode
        unsafe {
            match *self.state.get() {
                State::None => false,
                State::Done => false,
                State::Running => panic!("Portall::call() called reentrantly"),
                State::Waiting(func) => {
                    (*func)(val);
                    true
                }
            }
        }
    }

    pub fn wait_once<'a, R, F>(&'a self, mut func: F) -> impl Future<Output = R> + 'a
    where
        F: FnMut(T) -> R + 'a,
    {
        assert_thread_mode();

        async move {
            let signal = Signal::new();
            let mut result: MaybeUninit<R> = MaybeUninit::uninit();
            let mut call_func = |val: T| unsafe {
                let state = &mut *self.state.get();

                // Set state to Running while running the function to avoid reentrancy.
                *state = State::Running;
                result.as_mut_ptr().write(func(val));

                *state = State::Done;
                signal.signal(());
            };

            let func_ptr: *mut dyn FnMut(T) = &mut call_func as _;
            let func_ptr: *mut dyn FnMut(T) = unsafe { mem::transmute(func_ptr) };

            let _bomb = OnDrop::new(|| unsafe {
                let state = &mut *self.state.get();
                *state = State::None;
            });

            // safety: this runs from thread mode
            unsafe {
                let state = &mut *self.state.get();
                match state {
                    State::None => {}
                    _ => panic!("Multiple tasks waiting on same portal"),
                }
                *state = State::Waiting(func_ptr);
            }

            signal.wait().await;

            unsafe { result.assume_init() }
            // dropbomb sets self.state = None
        }
    }

    #[allow(unused)]
    pub fn wait_many<'a, R, F>(&'a self, mut func: F) -> impl Future<Output = R> + 'a
    where
        F: FnMut(T) -> Option<R> + 'a,
    {
        assert_thread_mode();

        async move {
            let signal = Signal::new();
            let mut result: MaybeUninit<R> = MaybeUninit::uninit();
            let mut call_func = |val: T| {
                unsafe {
                    let state = &mut *self.state.get();

                    let func_ptr = match *state {
                        State::Waiting(p) => p,
                        _ => unreachable!(),
                    };

                    // Set state to Running while running the function to avoid reentrancy.
                    *state = State::Running;

                    *state = match func(val) {
                        None => State::Waiting(func_ptr),
                        Some(res) => {
                            result.as_mut_ptr().write(res);
                            signal.signal(());
                            State::Done
                        }
                    };
                };
            };

            let func_ptr: *mut dyn FnMut(T) = &mut call_func as _;
            let func_ptr: *mut dyn FnMut(T) = unsafe { mem::transmute(func_ptr) };

            let _bomb = OnDrop::new(|| unsafe {
                let state = &mut *self.state.get();
                *state = State::None;
            });

            // safety: this runs from thread mode
            unsafe {
                let state = &mut *self.state.get();
                match *state {
                    State::None => {}
                    _ => panic!("Multiple tasks waiting on same portal"),
                }
                *state = State::Waiting(func_ptr);
            }

            signal.wait().await;

            unsafe { result.assume_init() }
            // dropbomb sets self.state = None
        }
    }
}
