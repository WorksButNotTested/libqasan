use {
    crate::hooks::{asan_load, size_t},
    core::ffi::{c_char, c_void},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strnlen"]
pub unsafe extern "C" fn strnlen(cs: *const c_char, maxlen: size_t) -> size_t {
    trace!("strnlen - cs: {:p}, maxlen: {:#x}", cs, maxlen);

    if maxlen == 0 {
        return 0;
    }

    if cs.is_null() {
        panic!("strnlen - cs is null");
    }

    let mut len = 0;
    while *cs.add(len) != 0 {
        len += 1;
    }

    if len < maxlen {
        asan_load(cs as *const c_void, len + 1);
        len
    } else {
        asan_load(cs as *const c_void, maxlen);
        maxlen
    }
}
