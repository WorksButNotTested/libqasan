use {
    crate::{asan_alloc, asan_dealloc, asan_load, hooks::size_t},
    core::{
        ffi::c_void,
        ptr::{copy_nonoverlapping, null_mut},
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
pub unsafe extern "C" fn realloc(p: *mut c_void, size: size_t) -> *mut c_void {
    trace!("realloc - p: {:p}, size: {:#x}", p, size);
    if p.is_null() {
        asan_alloc(size, 0)
    } else if size == 0 {
        asan_dealloc(p);
        null_mut()
    } else {
        asan_load(p, size);
        let q = asan_alloc(size, 0);
        unsafe { copy_nonoverlapping(p, q, size) };
        asan_dealloc(p);
        q
    }
}
