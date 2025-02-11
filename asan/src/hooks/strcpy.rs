use {
    crate::hooks::{asan_load, asan_store},
    core::{
        ffi::{c_char, c_void},
        ptr::copy,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strcpy"]
pub unsafe extern "C" fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    trace!("strcpy - dst: {:p}, src: {:p}", dst, src);

    if dst.is_null() {
        panic!("strcpy - dst is null");
    }

    if src.is_null() {
        panic!("strcpy - src is null");
    }

    let mut len = 0;
    while *src.add(len) != 0 {
        len += 1;
    }
    asan_load(src as *const c_void, len + 1);
    asan_store(dst as *const c_void, len + 1);
    copy(src, dst, len + 1);
    dst
}
