use {
    crate::hooks::{asan_store, size_t},
    core::{ffi::c_void, ptr::write_bytes},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[export_name = "patch_explicit_bzero"]
pub unsafe extern "C" fn explicit_bzero(s: *mut c_void, len: size_t) {
    trace!("explicit_bzero - s: {:p}, len: {:#x}", s, len);

    if len == 0 {
        return;
    }

    if s.is_null() {
        panic!("explicit_bzero - s is null");
    }

    asan_store(s, len);
    write_bytes(s, 0, len);
}
