use {
    crate::hooks::{asan_alloc, asan_dealloc, asan_load, size_t},
    core::{
        ffi::c_void,
        ptr::{copy_nonoverlapping, null_mut},
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn reallocarray(
    ptr: *mut c_void,
    nmemb: size_t,
    size: size_t,
) -> *mut c_void {
    trace!(
        "reallocarray - ptr: {:p}, nmemb: {:#x}, size: {:#x}",
        ptr,
        nmemb,
        size
    );
    match nmemb.checked_mul(size) {
        Some(size) => {
            if ptr.is_null() {
                asan_alloc(size, 0)
            } else if size == 0 {
                asan_dealloc(ptr);
                null_mut()
            } else {
                asan_load(ptr, size);
                let q = asan_alloc(size, 0);
                unsafe { copy_nonoverlapping(ptr, q, size) };
                asan_dealloc(ptr);
                q
            }
        }
        None => null_mut(),
    }
}
