use {
    crate::hooks::{asan_alloc, size_t},
    core::ffi::c_void,
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_malloc")]
pub unsafe extern "C" fn malloc(size: size_t) -> *mut c_void {
    trace!("malloc - size: {:#x}", size);
    asan_alloc(size, 0)
}
