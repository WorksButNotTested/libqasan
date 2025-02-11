use {crate::hooks::asan_dealloc, core::ffi::c_void, log::trace};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_free")]
pub unsafe extern "C" fn free(p: *mut c_void) {
    trace!("free - p: {:p}", p);
    asan_dealloc(p);
}
