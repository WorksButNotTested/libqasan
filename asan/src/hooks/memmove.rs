use {
    crate::{asan_load, asan_panic, asan_store, size_t},
    core::{
        ffi::{c_char, c_void},
        ptr::copy,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[export_name = "patch_memmove"]
pub unsafe extern "C" fn memmove(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void {
    trace!("memmove - dest: {:p}, src: {:p}, n: {:#x}", dest, src, n);

    if n == 0 {
        return dest;
    }

    if dest.is_null() {
        asan_panic(c"memmove - dest is null".as_ptr() as *const c_char);
    }

    if src.is_null() {
        asan_panic(c"memmove - src is null".as_ptr() as *const c_char);
    }

    asan_load(src, n);
    asan_store(dest, n);
    unsafe { copy(src, dest, n) };
    dest
}
