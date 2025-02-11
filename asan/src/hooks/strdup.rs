use {
    crate::hooks::{asan_alloc, asan_load},
    core::{
        ffi::{c_char, c_void},
        ptr::copy,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strdup"]
pub unsafe extern "C" fn strdup(cs: *const c_char) -> *mut c_char {
    trace!("strdup - cs: {:p}", cs);

    if cs.is_null() {
        panic!("strdup - cs is null");
    }

    let mut len = 0;
    while *cs.add(len) != 0 {
        len += 1;
    }
    asan_load(cs as *const c_void, len + 1);

    let dest = asan_alloc(len + 1, 0) as *mut c_char;
    copy(cs, dest, len + 1);
    dest
}
