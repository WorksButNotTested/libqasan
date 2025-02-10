use {
    crate::hooks::{asan_alloc, size_t},
    core::{ffi::c_void, ptr::null_mut},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn calloc(nobj: size_t, size: size_t) -> *mut c_void {
    trace!("calloc - nobj: {:#x}, size: {:#x}", nobj, size);
    match nobj.checked_mul(size) {
        Some(size) => asan_alloc(size, 0),
        None => null_mut(),
    }
}
