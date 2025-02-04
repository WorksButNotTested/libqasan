use {
    crate::{asan_get_size, hooks::size_t},
    core::ffi::c_void,
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn malloc_usable_size(ptr: *mut c_void) -> size_t {
    trace!("malloc_usable_size - ptr: {:p}", ptr);
    asan_get_size(ptr)
}
