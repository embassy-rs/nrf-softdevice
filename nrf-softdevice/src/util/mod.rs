#![macro_use]

mod signal;
pub use signal::*;
mod portal;
pub use portal::*;
mod drop_bomb;
pub use drop_bomb::*;
mod on_drop;
pub use on_drop::*;

use crate::raw;

/// Create a slice from a variable-length array in a BLE event.
///
/// This function is a workaround for UB in __IncompleteArrayField
/// see https://github.com/rust-lang/rust-bindgen/issues/1892
/// see https://github.com/rust-lang/unsafe-code-guidelines/issues/134
#[allow(unused)]
pub(crate) unsafe fn get_flexarray<T>(
    orig_ptr: *const raw::ble_evt_t,
    array: &raw::__IncompleteArrayField<T>,
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
