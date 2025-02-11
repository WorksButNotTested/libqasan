use {
    crate::hooks::{asan_get_size, size_t},
    core::ffi::c_void,
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_malloc_usable_size")]
pub unsafe extern "C" fn malloc_usable_size(ptr: *mut c_void) -> size_t {
    trace!("malloc_usable_size - ptr: {:p}", ptr);
    if ptr.is_null() {
        0
    } else {
        asan_get_size(ptr)
    }
}
