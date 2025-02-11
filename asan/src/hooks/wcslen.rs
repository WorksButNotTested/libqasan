use {
    crate::hooks::{asan_load, size_t, wchar_t},
    core::ffi::c_void,
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_wcslen"]
pub unsafe extern "C" fn wcslen(buf: *const wchar_t) -> size_t {
    trace!("wcslen - buf: {:p}", buf);

    if buf.is_null() {
        panic!("wcslen - buf is null");
    }

    let mut len = 0;
    while *buf.add(len) != 0 {
        len += 1;
    }
    asan_load(buf as *const c_void, size_of::<wchar_t>() * (len + 1));
    len
}
