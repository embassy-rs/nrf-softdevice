#![macro_use]

mod macros;

mod signal;
pub use signal::*;
mod portal;
pub use portal::*;
mod waker_store;
pub use waker_store::*;
mod drop_bomb;
pub use drop_bomb::*;

pub(crate) use defmt::{debug, error, info, intern, trace, warn};

use crate::raw;

pub trait Dewrap<T> {
    /// dewrap = defmt unwrap
    fn dewrap(self) -> T;

    /// dexpect = defmt expect
    fn dexpect<M: defmt::Format>(self, msg: M) -> T;

    fn dewarn<M: defmt::Format>(self, msg: M) -> Self;
}

impl<T> Dewrap<T> for Option<T> {
    fn dewrap(self) -> T {
        match self {
            Some(t) => t,
            None => depanic!("unwrap failed: enum is none"),
        }
    }

    fn dexpect<M: defmt::Format>(self, msg: M) -> T {
        match self {
            Some(t) => t,
            None => depanic!("unexpected None: {:?}", msg),
        }
    }

    fn dewarn<M: defmt::Format>(self, msg: M) -> Self {
        if self.is_none() {
            warn!("{:?} is none", msg);
        }
        self
    }
}

impl<T, E: defmt::Format> Dewrap<T> for Result<T, E> {
    fn dewrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => depanic!("unwrap failed: {:?}", e),
        }
    }

    fn dexpect<M: defmt::Format>(self, msg: M) -> T {
        match self {
            Ok(t) => t,
            Err(e) => depanic!("unexpected error: {:?}: {:?}", msg, e),
        }
    }

    fn dewarn<M: defmt::Format>(self, msg: M) -> Self {
        if let Err(e) = &self {
            warn!("{:?} err: {:?}", msg, e);
        }
        self
    }
}

/// Create a slice from a variable-length array in a BLE event.
///
/// This function is a workaround for UB in __IncompleteArrayField
/// see https://github.com/rust-lang/rust-bindgen/issues/1892
/// see https://github.com/rust-lang/unsafe-code-guidelines/issues/134
pub(crate) unsafe fn get_flexarray<T>(
    orig_ptr: *const raw::ble_evt_t,
    array: &raw::__IncompleteArrayField<T>,
    count: usize,
) -> &[T] {
    let offs = array.as_ptr() as usize - orig_ptr as usize;
    let sanitized_ptr = (orig_ptr as *const u8).add(offs) as *const T;
    core::slice::from_raw_parts(sanitized_ptr, count)
}

/// Create a slice from a variable-length array in a BLE event.
///
/// This function is a workaround for UB in __IncompleteArrayField
/// see https://github.com/rust-lang/rust-bindgen/issues/1892
/// see https://github.com/rust-lang/unsafe-code-guidelines/issues/134
pub(crate) unsafe fn get_flexarray2<T>(
    orig_ptr: *const raw::ble_evt_t,
    array: &[T; 0],
    count: usize,
) -> &[T] {
    let offs = array.as_ptr() as usize - orig_ptr as usize;
    let sanitized_ptr = (orig_ptr as *const u8).add(offs) as *const T;
    core::slice::from_raw_parts(sanitized_ptr, count)
}

/// Get a &T from a __BindgenUnionField<T> in a BLE event.
///
/// This function is a workaround for UB in __BindgenUnionField
/// see https://github.com/rust-lang/rust-bindgen/issues/1892
/// see https://github.com/rust-lang/unsafe-code-guidelines/issues/134
pub(crate) unsafe fn get_union_field<T>(
    orig_ptr: *const raw::ble_evt_t,
    member: &raw::__BindgenUnionField<T>,
) -> &T {
    let offs = member as *const _ as usize - orig_ptr as usize;
    let sanitized_ptr = (orig_ptr as *const u8).add(offs) as *const T;
    &*sanitized_ptr
}
