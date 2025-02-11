use {
    crate::hooks::asan_load,
    core::{
        ffi::{c_char, c_void},
        ptr::copy,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strcat"]
pub unsafe extern "C" fn strcat(s: *mut c_char, ct: *const c_char) -> *mut c_char {
    trace!("strcat - s: {:p}, ct: {:p}", s, ct);

    if s.is_null() {
        panic!("strcat - s is null");
    }

    if ct.is_null() {
        panic!("strcat - ct is null");
    }

    let mut s_len = 0;
    while *s.add(s_len) != 0 {
        s_len += 1;
    }
    let mut ct_len = 0;
    while *ct.add(ct_len) != 0 {
        ct_len += 1;
    }
    asan_load(s as *const c_void, s_len + 1);
    asan_load(ct as *const c_void, ct_len + 1);
    copy(ct, s.add(s_len), ct_len + 1);
    s
}
