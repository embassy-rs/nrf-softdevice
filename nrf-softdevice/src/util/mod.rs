#![macro_use]

mod signal;
pub use signal::*;
mod portal;
pub use portal::*;
mod waker_store;
pub use waker_store::*;
mod drop_bomb;
pub use drop_bomb::*;
mod on_drop;
pub use on_drop::*;

use crate::raw;
pub use defmt::{
    assert, assert_eq, assert_ne, debug, debug_assert, debug_assert_eq, debug_assert_ne, error,
    info, intern, panic, trace, unimplemented, unreachable, unwrap, warn,
};

pub(crate) struct BoundedLifetime;

impl BoundedLifetime {
    pub(crate) unsafe fn deref<T>(&self, ptr: *const T) -> &T {
        &*ptr
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
