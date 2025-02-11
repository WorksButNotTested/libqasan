use {
    crate::hooks::{asan_load, asan_store, size_t},
    core::{ffi::c_void, ptr::copy_nonoverlapping},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_memcpy"]
pub unsafe extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void {
    trace!("memcpy - dest: {:p}, src: {:p}, n: {:#x}", dest, src, n);

    if n == 0 {
        return dest;
    }

    if dest.is_null() {
        panic!("memcpy - dest is null");
    }

    if src.is_null() {
        panic!("memcpy - src is null");
    }

    let src_end = src.add(n);
    let dest_end = dest.add(n) as *const c_void;
    if src_end > dest && dest_end > src {
        panic!("memcpy - overlap");
    }

    asan_load(src, n);
    asan_store(dest, n);
    unsafe { copy_nonoverlapping(src, dest, n) };
    dest
}
