use {
    crate::hooks::{asan_alloc, asan_dealloc, asan_get_size, asan_load, size_t},
    core::{
        ffi::c_void,
        ptr::{copy_nonoverlapping, null_mut},
    },
    log::trace,
};

/// # Safety
/// See man pages
#[no_mangle]
#[cfg_attr(feature = "test", export_name = "patch_reallocarray")]
pub unsafe extern "C" fn reallocarray(
    ptr: *mut c_void,
    nmemb: size_t,
    size: size_t,
) -> *mut c_void {
    trace!(
        "reallocarray - ptr: {:p}, nmemb: {:#x}, size: {:#x}",
        ptr,
        nmemb,
        size
    );
    match nmemb.checked_mul(size) {
        Some(size) => {
            if ptr.is_null() {
                asan_alloc(size, 0)
            } else if size == 0 {
                asan_dealloc(ptr);
                null_mut()
            } else {
                let old_size = asan_get_size(ptr);
                asan_load(ptr, size);
                let q = asan_alloc(size, 0);
                let min = old_size.min(size);
                unsafe { copy_nonoverlapping(ptr as *const u8, q as *mut u8, min) };
                asan_dealloc(ptr);
                q
            }
        }
        None => panic!("reallocarray - size would overflow"),
    }
}
