use {
    crate::hooks::{asan_alloc, asan_load, size_t},
    core::{
        ffi::{c_char, c_void},
        ptr::copy,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strndup"]
pub unsafe extern "C" fn strndup(cs: *const c_char, n: size_t) -> *mut c_char {
    trace!("strndup - cs: {:p}, n: {:#x}", cs, n);

    if cs.is_null() && n != 0 {
        panic!("strndup - cs is null");
    }

    let mut len = 0;
    while len < n && *cs.add(len) != 0 {
        len += 1;
    }
    asan_load(cs as *const c_void, len + 1);

    let dest = asan_alloc(len + 1, 0) as *mut c_char;
    copy(cs, dest, len + 1);
    *dest.add(len) = 0;
    dest
}
