use {
    crate::hooks::{asan_load, size_t},
    core::ffi::{c_char, c_void},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_strlen"]
pub unsafe extern "C" fn strlen(cs: *const c_char) -> size_t {
    trace!("strlen - cs: {:p}", cs);

    if cs.is_null() {
        panic!("strlen - cs is null");
    }

    let mut len = 0;
    while *cs.add(len) != 0 {
        len += 1;
    }
    asan_load(cs as *const c_void, len + 1);
    len
}
