use {
    crate::hooks::{asan_alloc, asan_page_size, size_t},
    core::ffi::c_void,
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_pvalloc")]
pub unsafe extern "C" fn pvalloc(size: size_t) -> *mut c_void {
    trace!("pvalloc - size: {:#x}", size);
    let page_size = asan_page_size();
    let aligned_size = (size + page_size - 1) & (page_size - 1);
    asan_alloc(aligned_size, page_size)
}
