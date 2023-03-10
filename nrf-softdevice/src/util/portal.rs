use core::cell::RefCell;
use core::mem;
use core::mem::MaybeUninit;

use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::signal::Signal;

use crate::util::OnDrop;

/// Utility to call a closure across tasks.
pub struct Portal<T> {
    #[cfg(feature = "usable-from-interrupts")]
    state: Mutex<CriticalSectionRawMutex, RefCell<State<T>>>,
    #[cfg(not(feature = "usable-from-interrupts"))]
    state: Mutex<ThreadModeRawMutex, RefCell<State<T>>>,
}

enum State<T> {
    None,
    Running,
    Waiting(*mut dyn FnMut(T)),
    Done,
}

unsafe impl<T> Send for Portal<T> {}

unsafe impl<T> Sync for Portal<T> {}

impl<T> Portal<T> {
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(RefCell::new(State::None)),
        }
    }

    pub fn call(&self, val: T) -> bool {
        let maybe_func = self.state.lock(|state| match *state.borrow() {
            State::None => None,
            State::Done => None,
            State::Running => panic!("Portal::call() called reentrantly"),
            State::Waiting(func) => Some(func),
        });

        // re-entrant calling possible here. Acceptable because Portal::call() panics.

        if let Some(ptr) = maybe_func {
            // Safety: This is transmuted from a FnMut, and therefore valid
            unsafe { (*ptr)(val) };
        }
        true
    }

    pub async fn wait_once<'a, R, F>(&'a self, mut func: F) -> R
        where
            F: FnMut(T) -> R + 'a,
    {
        let signal = Signal::<CriticalSectionRawMutex, _>::new();
        let mut result: MaybeUninit<R> = MaybeUninit::uninit();

        let mut call_func = |val: T| unsafe {
            self.state.lock(|state| {
                let mut state = state.borrow_mut();
                // Set state to Running while running the function to avoid reentrancy.
                *state = State::Running;
                result.as_mut_ptr().write(func(val));

                *state = State::Done;
                signal.signal(());
            });
        };

        let func_ptr: *mut dyn FnMut(T) = &mut call_func as _;
        let func_ptr: *mut dyn FnMut(T) = unsafe { mem::transmute(func_ptr) };

        let _bomb = OnDrop::new(|| {
            self.state.lock(|state| {
                let mut state = state.borrow_mut();
                *state = State::None;
            });
        });

        // safety: this runs from thread mode
        self.state.lock(|state| {
            let mut state = state.borrow_mut();
            match *state {
                State::None => {}
                _ => panic!("Multiple tasks waiting on same portal"),
            }
            *state = State::Waiting(func_ptr);
        });

        signal.wait().await;

        unsafe { result.assume_init() }
        // dropbomb sets self.state = None
    }

    #[allow(unused)]
    pub async fn wait_many<'a, R, F>(&'a self, mut func: F) -> R
        where
            F: FnMut(T) -> Option<R> + 'a,
    {
        let signal = Signal::<CriticalSectionRawMutex, _>::new();
        let mut result: MaybeUninit<R> = MaybeUninit::uninit();
        let mut call_func = |val: T| {
            self.state.lock(|state| {
                let mut state = state.borrow_mut();

                let func_ptr = match *state {
                    State::Waiting(p) => p,
                    _ => unreachable!(),
                };

                // Set state to Running while running the function to avoid reentrancy.
                *state = State::Running;

                *state = match func(val) {
                    None => State::Waiting(func_ptr),
                    Some(res) => {
                        unsafe {
                            result.as_mut_ptr().write(res);
                        }
                        signal.signal(());
                        State::Done
                    }
                };
            });
        };

        let func_ptr: *mut dyn FnMut(T) = &mut call_func as _;
        let func_ptr: *mut dyn FnMut(T) = unsafe { mem::transmute(func_ptr) };

        let _bomb = OnDrop::new(|| {
            self.state.lock(|state| {
                let mut state = state.borrow_mut();
                *state = State::None;
            });
        });

        // safety: this runs from thread mode
        self.state.lock(|state| {
            let mut state = state.borrow_mut();
            match *state {
                State::None => {}
                _ => panic!("Multiple tasks waiting on same portal"),
            }
            *state = State::Waiting(func_ptr);
        });

        signal.wait().await;

        unsafe { result.assume_init() }
        // dropbomb sets self.state = None
    }
}
