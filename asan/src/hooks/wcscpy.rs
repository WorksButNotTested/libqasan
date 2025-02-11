use {
    crate::hooks::{asan_load, asan_store, wchar_t},
    core::{ffi::c_void, ptr::copy},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_wcscpy"]
pub unsafe extern "C" fn wcscpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t {
    trace!("wcscpy - dst: {:p}, src: {:p}", dst, src);

    if dst.is_null() {
        panic!("wcscpy - dst is null");
    }

    if src.is_null() {
        panic!("wcscpy - src is null");
    }

    let mut len = 0;
    while *src.add(len) != 0 {
        len += 1;
    }
    asan_load(src as *const c_void, size_of::<wchar_t>() * (len + 1));
    asan_store(dst as *const c_void, size_of::<wchar_t>() * (len + 1));
    copy(src, dst, len + 1);
    dst
}
