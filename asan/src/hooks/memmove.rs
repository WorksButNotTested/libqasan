use {
    crate::hooks::{asan_load, asan_store, size_t},
    core::{ffi::c_void, ptr::copy},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_memmove"]
pub unsafe extern "C" fn memmove(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void {
    trace!("memmove - dest: {:p}, src: {:p}, n: {:#x}", dest, src, n);

    if n == 0 {
        return dest;
    }

    if dest.is_null() {
        panic!("memmove - dest is null");
    }

    if src.is_null() {
        panic!("memmove - src is null");
    }

    asan_load(src, n);
    asan_store(dest, n);
    unsafe { copy(src, dest, n) };
    dest
}
