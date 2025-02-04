use {
    crate::{asan_alloc, asan_page_size, hooks::size_t},
    core::ffi::c_void,
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn valloc(size: size_t) -> *mut c_void {
    trace!("valloc - size: {:#x}", size);
    asan_alloc(size, asan_page_size())
}
