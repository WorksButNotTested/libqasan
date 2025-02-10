use {crate::hooks::asan_dealloc, core::ffi::c_void, log::trace};

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn free(p: *mut c_void) {
    trace!("free - p: {:p}", p);
    asan_dealloc(p);
}
