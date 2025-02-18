use {
    crate::{asan_panic, asan_store, size_t},
    core::{
        ffi::{c_char, c_void},
        ptr::write_bytes,
    },
    log::trace,
};

/// # Safety
/// See man pages
#[export_name = "patch_bzero"]
pub unsafe extern "C" fn bzero(s: *mut c_void, len: size_t) {
    trace!("bzero - s: {:p}, len: {:#x}", s, len);

    if len == 0 {
        return;
    }

    if s.is_null() {
        asan_panic(c"bzero - s is null".as_ptr() as *const c_char);
    }

    asan_store(s, len);
    write_bytes(s, 0, len);
}
