use {
    crate::{asan_alloc, asan_dealloc, asan_get_size, asan_load, size_t},
    core::{
        ffi::c_void,
        ptr::{copy_nonoverlapping, null_mut},
    },
    log::trace,
};

/// # Safety
/// See man pages
#[cfg_attr(not(feature = "test"), no_mangle)]
#[cfg_attr(feature = "test", export_name = "patch_realloc")]
pub unsafe extern "C" fn realloc(p: *mut c_void, size: size_t) -> *mut c_void {
    trace!("realloc - p: {:p}, size: {:#x}", p, size);
    if p.is_null() && size == 0 {
        null_mut()
    } else if p.is_null() {
        asan_alloc(size, 0)
    } else if size == 0 {
        asan_dealloc(p);
        null_mut()
    } else {
        let old_size = asan_get_size(p);
        asan_load(p, old_size);
        let q = asan_alloc(size, 0);
        let min = old_size.min(size);
        unsafe { copy_nonoverlapping(p as *const u8, q as *mut u8, min) };
        asan_dealloc(p);
        q
    }
}
