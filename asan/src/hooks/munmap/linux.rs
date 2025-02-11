use {
    crate::hooks::{asan_untrack, size_t},
    core::ffi::{c_int, c_void},
    log::trace,
    rustix::mm::munmap as rmunmap,
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_munmap")]
pub unsafe extern "C" fn munmap(addr: *mut c_void, len: size_t) -> c_int {
    trace!("munmap - addr: {:p}, len: {:#x}", addr, len);

    if rmunmap(addr, len).is_ok() {
        asan_untrack(addr);
        0
    } else {
        -1
    }
}
