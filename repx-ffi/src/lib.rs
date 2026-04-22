// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
//
// repx-ffi — stub C ABI surface for the RepX engine.
// Full extern "C" wrappers (`repx_read`, `repx_convert`,
// `repx_describe_json`, `repx_find_json`) plus panic-safe
// `catch_unwind` boundaries land in task-133 G8 follow-up.

#![deny(unsafe_op_in_unsafe_fn)]

/// Sentinel return code emitted by FFI shims that have not been
/// implemented yet. Real codes (REPX_FFI_PANIC, REPX_FFI_NAME_TOO_LONG,
/// REPX_FFI_BAD_NUL, REPX_FFI_BAD_UTF8) land in G8.
pub const REPX_FFI_NOT_IMPLEMENTED: i32 = -127;

/// Panic-return sentinel reserved for the catch_unwind boundary (G8).
pub const REPX_FFI_PANIC: i32 = -1;

/// Stub describe_json — always returns NotImplemented.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string or null. The function
/// does not currently dereference the pointer.
#[no_mangle]
pub unsafe extern "C" fn repx_describe_json(
    _name: *const std::os::raw::c_char,
    _out_buf: *mut std::os::raw::c_char,
    _out_cap: usize,
) -> i32 {
    REPX_FFI_NOT_IMPLEMENTED
}
