use {
    crate::hooks::{asan_load, asan_store, size_t},
    core::{ffi::c_void, ptr::copy_nonoverlapping},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_mempcpy"]
pub unsafe extern "C" fn mempcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void {
    trace!("mempcpy - dest: {:p}, src: {:p}, n: {:#x}", dest, src, n);

    if n == 0 {
        return dest;
    }

    if dest.is_null() {
        panic!("mempcpy - dest is null");
    }

    if src.is_null() {
        panic!("mempcpy - src is null");
    }

    asan_load(src, n);
    asan_store(dest, n);
    unsafe { copy_nonoverlapping(src, dest, n) };
    dest.add(n)
}
