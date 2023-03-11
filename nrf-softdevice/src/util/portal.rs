use core::cell::RefCell;
use core::mem;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

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

struct State<T>(Option<NonNull<dyn FnMut(T, &mut State<T>)>>);

unsafe impl<T> Send for Portal<T> {}

unsafe impl<T> Sync for Portal<T> {}

impl<T> Portal<T> {
    const INIT: Self = Portal {
        state: Mutex::new(RefCell::new(State(None))),
    };
    pub const fn new() -> Self {
        Self::INIT
    }

    /// Execute the closure that the portal currently holds onto, if one is present.
    ///
    /// # Considerations
    ///
    /// This will block until the closure contained within the portal (if any) has finished executing.
    /// This will be entirely done within a critical section, and can therefore *not be preceeded
    /// by anything*. Be aware of this when calling this function.
    ///
    pub fn call(&self, val: T) -> bool {
        self.state.lock(|state| {
            let mut state = state.borrow_mut();
            if let Some(ptr) = state.0 {
                // Safety: This is transmuted from a FnMut, and therefore valid
                unsafe { (*ptr.as_ptr())(val, &mut *state) };
                true
            } else {
                false
            }
        })
    }

    /// Wait until the portal is called once using the [Portal::call()] function.
    ///
    /// The closure will be called with the parameter provided to [Portal::call()].
    /// The closure's result will be returned once it is available.
    ///
    ///
    /// # Panics
    ///
    /// When a closure is already waiting to be executed on this portal, this
    /// will panic
    ///
    /// # Considerations
    ///
    /// [Portal::call()] will block until the closure finished executing, which will be done within
    /// a critical section. Therefore, even with concurrency frameworks and such, the closure will
    /// lock the application for its run duration. So, the caller is responsible for creating
    /// closures with short enough execution times to not massively disrupt the control flow of any
    /// application, especially when this is used from a library
    pub async fn wait_once<'a, R, F>(&'a self, mut func: F) -> R
    where
        F: FnMut(T) -> R + 'a,
    {
        let signal = Signal::<CriticalSectionRawMutex, _>::new();
        let mut result: MaybeUninit<R> = MaybeUninit::uninit();

        let call_func = |val: T, state: &mut State<T>| unsafe {
            result.as_mut_ptr().write(func(val));

            signal.signal(());

            *state = State(None)
            // state gets dropped here, which allows calling the function again
        };

        // If the future gets cancelled from the outside, this gets dropped,
        // and resets the state of the portal to None
        let _bomb = OnDrop::new(|| {
            self.state.lock(|state| *(state.borrow_mut()) = State(None));
        });

        self.set_function_pointer(call_func);

        signal.wait().await;

        unsafe { result.assume_init() }
    }

    /// Wait until the portal is called once the [Portal::call()] function, and the closure
    /// returns `Some(T)`.
    ///
    /// The closure will be called with the parameter provided to [Portal::call()].
    /// The closure's result will be returned once it is available
    /// As long as the closure returns `None`, the next call to [Portal::call()] will once again
    /// execute that closure. The future will only complete after the closure returns `Some(T)`.
    ///
    /// # Panics
    ///
    /// When a closure is already waiting to be executed on this portal, this
    /// will panic
    ///
    /// # Considerations
    ///
    /// [Portal::call()] will block until the closure finished executing, which will be done within
    /// a critical section. Therefore, even with concurrency frameworks and such, the closure will
    /// lock the application for its run duration. So, the caller is responsible for creating
    /// closures with short enough execution times to not massively disrupt the control flow of any
    /// application, especially when this is used from a library
    #[allow(unused)]
    pub async fn wait_many<'a, R, F>(&'a self, mut func: F) -> R
    where
        F: FnMut(T) -> Option<R> + 'a,
    {
        let signal = Signal::<CriticalSectionRawMutex, _>::new();
        let mut result: MaybeUninit<R> = MaybeUninit::uninit();
        let mut call_func = |val: T, state: &mut State<T>| {
            let func_ptr = match *state {
                State(Some((p))) => p,
                _ => unreachable!(),
            };

            if let Some(res) = func(val) {
                unsafe {
                    result.as_mut_ptr().write(res);
                }
                signal.signal(());
                *state = State(None)
            }
            // state gets dropped here, which allows calling the function again
        };

        // If the future gets cancelled from the outside, this gets dropped,
        // and resets the state of the portal to None
        let _bomb = OnDrop::new(|| {
            self.state.lock(|mut state| *(state.borrow_mut()) = State(None));
        });

        self.set_function_pointer(call_func);

        signal.wait().await;

        unsafe { result.assume_init() }
    }

    /// Utility function for setting the current waiting function pointer
    ///
    /// # Panics
    ///
    /// This panics when [self.state] is not `State(None)`, and therefore there
    /// is currently a task waiting on the portal.
    fn set_function_pointer(&self, mut call_func: impl FnMut(T, &mut State<T>)) {
        let func_ptr: *mut dyn FnMut(T, &mut State<T>) = &mut call_func as _;

        // Safety: Needs to be validated!!!
        let func_ptr: *mut dyn FnMut(T, &mut State<T>) = unsafe { mem::transmute(func_ptr) };

        self.state.lock(|state| {
            let mut state = state.borrow_mut();
            match *state {
                State(None) => {}
                _ => panic!("Multiple tasks waiting on same portal"),
            }
            *state = State(NonNull::new(func_ptr));
        });
    }
}
