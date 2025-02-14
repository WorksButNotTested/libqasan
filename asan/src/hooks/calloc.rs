use {
    crate::hooks::{asan_alloc, asan_panic, size_t},
    core::{
        ffi::{c_char, c_void},
        ptr::{null_mut, write_bytes},
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_calloc")]
pub unsafe extern "C" fn calloc(nobj: size_t, size: size_t) -> *mut c_void {
    trace!("calloc - nobj: {:#x}, size: {:#x}", nobj, size);
    match nobj.checked_mul(size) {
        Some(0) => null_mut(),
        Some(size) => {
            let ptr = asan_alloc(size, 0);
            unsafe { write_bytes(ptr, 0, size) };
            ptr
        }
        None => {
            asan_panic(c"calloc - size would overflow".as_ptr() as *const c_char);
        }
    }
}
