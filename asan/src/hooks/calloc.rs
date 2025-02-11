use {
    crate::hooks::{asan_alloc, size_t},
    core::{ffi::c_void, ptr::write_bytes},
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_calloc")]
pub unsafe extern "C" fn calloc(nobj: size_t, size: size_t) -> *mut c_void {
    trace!("calloc - nobj: {:#x}, size: {:#x}", nobj, size);
    match nobj.checked_mul(size) {
        Some(size) => {
            let ptr = asan_alloc(size, 0);
            unsafe { write_bytes(ptr, 0, size) };
            ptr
        }
        None => {
            panic!("calloc - size would overflow");
        }
    }
}
