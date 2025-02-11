use {
    crate::hooks::{asan_alloc, asan_page_size, size_t},
    core::ffi::c_void,
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_valloc")]
pub unsafe extern "C" fn valloc(size: size_t) -> *mut c_void {
    trace!("valloc - size: {:#x}", size);
    asan_alloc(size, asan_page_size())
}
